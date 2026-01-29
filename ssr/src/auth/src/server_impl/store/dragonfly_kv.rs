use super::{KVError, KVStore};
use futures::StreamExt;
use redis::aio::MultiplexedConnection;
use redis::sentinel::SentinelClient;
use redis::sentinel::SentinelClientBuilder;
use redis::sentinel::SentinelServerType;
use redis::AsyncConnectionConfig;
use redis::Client;
use redis::ClientTlsConfig;
use redis::ConnectionAddr;
use redis::{AsyncCommands, RedisError};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

pub const REDIS_SENTINEL_PORT: u16 = 26379;
pub const SENTINEL_SERVICE_NAME: &str = "mymaster";

pub const TEST_KEY_PREFIX: &str = "test";

const SENTINEL_RECONNECT_DELAY: Duration = Duration::from_secs(1);

pub fn format_to_dragonfly_key(key_prefix: &str, key: &str) -> String {
    format!("{key_prefix}:{key}")
}

pub fn normalize_pem(pem: String) -> Vec<u8> {
    let normalized = pem
        .replace("\\n", "\n")
        .replace("\\r\\n", "\n")
        .replace("\\r", "")
        .replace("\r\n", "\n")
        .replace("\r", "")
        .trim()
        .to_string();
    if normalized.ends_with('\n') {
        normalized.into_bytes()
    } else {
        format!("{normalized}\n").into_bytes()
    }
}

pub fn get_ca_cert_pem() -> Result<Vec<u8>, anyhow::Error> {
    Ok(normalize_pem(
        std::env::var("DRAGONFLY_CA_CERT").expect("DRAGONFLY_CA_CERT env var not set"),
    ))
}

pub fn get_client_cert_pem() -> Result<Vec<u8>, anyhow::Error> {
    Ok(normalize_pem(
        std::env::var("DRAGONFLY_CLIENT_CERT").expect("DRAGONFLY_CLIENT_CERT env var not set"),
    ))
}

pub fn get_client_key_pem() -> Result<Vec<u8>, anyhow::Error> {
    Ok(normalize_pem(
        std::env::var("DRAGONFLY_CLIENT_KEY").expect("DRAGONFLY_CLIENT_KEY env var not set"),
    ))
}

fn build_tls_certs(
    ca_cert_bytes: Vec<u8>,
    client_cert_bytes: Vec<u8>,
    client_key_bytes: Vec<u8>,
) -> redis::TlsCertificates {
    redis::TlsCertificates {
        client_tls: Some(ClientTlsConfig {
            client_cert: client_cert_bytes,
            client_key: client_key_bytes,
        }),
        root_cert: Some(ca_cert_bytes),
    }
}

fn get_hosts_from_env() -> Vec<String> {
    let hosts_str = std::env::var("DRAGONFLY_HOSTS")
        .expect("DRAGONFLY_HOSTS environment variable not set")
        .trim()
        .to_string();

    hosts_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Simple connection pool - just wraps SentinelConnectionManager
/// MultiplexedConnection handles multiplexing internally, so we just need one connection
pub struct DragonflyPool {
    connection_manager: Arc<SentinelConnectionManager>,
    /// Cached connection - MultiplexedConnection is cheap to clone
    cached_conn: RwLock<Option<MultiplexedConnection>>,
}

impl DragonflyPool {
    pub fn new(connection_manager: Arc<SentinelConnectionManager>) -> Arc<Self> {
        Arc::new(Self {
            connection_manager,
            cached_conn: RwLock::new(None),
        })
    }

    /// Get a connection - returns cached or creates new one
    pub async fn get(&self) -> std::result::Result<MultiplexedConnection, RedisError> {
        // Try cached connection first
        {
            let guard = self.cached_conn.read().await;
            if let Some(conn) = guard.as_ref() {
                return Ok(conn.clone());
            }
        }

        // Create new connection
        let conn = self.connection_manager.connect().await?;

        // Cache it
        {
            let mut guard = self.cached_conn.write().await;
            *guard = Some(conn.clone());
        }

        Ok(conn)
    }

    /// Get a validated connection (verified with PING)
    pub async fn get_validated(&self) -> std::result::Result<MultiplexedConnection, RedisError> {
        // Try cached connection first
        {
            let guard = self.cached_conn.read().await;
            if let Some(conn) = guard.as_ref() {
                let mut conn_clone = conn.clone();
                if let Ok(pong) = redis::cmd("PING")
                    .query_async::<String>(&mut conn_clone)
                    .await
                {
                    if pong == "PONG" {
                        return Ok(conn_clone);
                    }
                }
            }
        }

        // Cached connection is stale, create new one
        let conn = self.connection_manager.connect().await?;

        // Cache it
        {
            let mut guard = self.cached_conn.write().await;
            *guard = Some(conn.clone());
        }

        Ok(conn)
    }

    /// Invalidate cached connection (call after connection errors)
    pub async fn invalidate(&self) {
        let mut guard = self.cached_conn.write().await;
        *guard = None;
    }

    pub async fn execute_with_retry<F, Fut, T>(
        &self,
        mut operation: F,
    ) -> std::result::Result<T, RedisError>
    where
        F: FnMut(MultiplexedConnection) -> Fut,
        Fut: std::future::Future<Output = std::result::Result<T, RedisError>>,
    {
        let conn = self.get().await?;
        match operation(conn).await {
            Ok(result) => Ok(result),
            Err(e)
                if e.is_connection_dropped()
                    || e.is_timeout()
                    || e.is_connection_refusal()
                    || e.is_io_error() =>
            {
                tracing::warn!(error = %e, "Connection error detected, invalidating cache and retrying");
                self.invalidate().await;

                let fresh_conn = self.get().await?;
                operation(fresh_conn).await
            }
            Err(e) => Err(e),
        }
    }
}

// ============================================================================
// Sentinel Connection Manager
// ============================================================================

#[derive(Clone)]
pub struct SentinelConnectionManager {
    sentinel_client: Arc<RwLock<SentinelClient>>,
    master_name: String,
    /// Cached master client - invalidated only by pub/sub failover events
    cached_master: Arc<RwLock<Option<Client>>>,
}

impl SentinelConnectionManager {
    pub fn new(
        sentinel_client: SentinelClient,
        master_name: String,
    ) -> Result<Self, anyhow::Error> {
        Ok(Self {
            sentinel_client: Arc::new(RwLock::new(sentinel_client)),
            master_name,
            cached_master: Arc::new(RwLock::new(None)),
        })
    }

    /// Called when pub/sub detects a failover - clears the cached master
    async fn on_failover_detected(&self) {
        tracing::warn!("Failover detected! Invalidating master cache");
        let mut cache = self.cached_master.write().await;
        *cache = None;
    }

    /// Get master client - uses cache if available, otherwise queries Sentinel
    async fn get_master_client(&self) -> std::result::Result<Client, RedisError> {
        // Fast path: return cached client if available
        {
            let cache = self.cached_master.read().await;
            if let Some(ref client) = *cache {
                return Ok(client.clone());
            }
        }

        // Slow path: query Sentinel and cache the result
        let mut cache = self.cached_master.write().await;

        // Double-check after acquiring write lock
        if let Some(ref client) = *cache {
            return Ok(client.clone());
        }

        let mut sentinel = self.sentinel_client.write().await;
        let client = sentinel.get_client()?;

        // Log the discovered master for debugging
        let connection_info = client.get_connection_info();
        let (host, port) = match connection_info.addr() {
            redis::ConnectionAddr::Tcp(h, p) => (h.clone(), *p),
            redis::ConnectionAddr::TcpTls { host, port, .. } => (host.clone(), *port),
            _ => ("unknown".to_string(), 0),
        };

        tracing::info!(host = %host, port = port, "Discovered master from Sentinel");

        *cache = Some(client.clone());
        Ok(client)
    }

    pub async fn start_failover_listener(self: Arc<Self>, tls_certs: redis::TlsCertificates) {
        let hosts = get_hosts_from_env();

        if hosts.is_empty() {
            tracing::error!("No Sentinel hosts configured, failover listener disabled");
            return;
        }

        tracing::info!(
            hosts = ?hosts,
            "Starting Sentinel failover listener"
        );

        loop {
            for host in &hosts {
                tracing::debug!(host = %host, "Attempting Sentinel pub/sub connection");

                match self
                    .subscribe_to_sentinel(host.clone(), tls_certs.clone())
                    .await
                {
                    Ok(()) => {
                        tracing::warn!(
                            host = %host,
                            "Sentinel subscription ended unexpectedly"
                        );
                    }
                    Err(e) => {
                        tracing::error!(
                            host = %host,
                            error = %e,
                            "Sentinel subscription failed"
                        );
                    }
                }

                tokio::time::sleep(Duration::from_millis(500)).await;
            }

            tracing::warn!(
                "All Sentinel connections failed, retrying in {:?}",
                SENTINEL_RECONNECT_DELAY
            );
            tokio::time::sleep(SENTINEL_RECONNECT_DELAY).await;
        }
    }

    async fn subscribe_to_sentinel(
        &self,
        host: String,
        tls_certs: redis::TlsCertificates,
    ) -> std::result::Result<(), RedisError> {
        let url = format!("rediss://{host}:{REDIS_SENTINEL_PORT}");
        let client = redis::Client::build_with_tls(url, tls_certs)?;

        let mut pubsub = client.get_async_pubsub().await?;

        pubsub.subscribe("+switch-master").await?;
        pubsub.subscribe("+sdown").await?;
        pubsub.subscribe("+odown").await?;
        pubsub.subscribe("-sdown").await?;
        pubsub.subscribe("-odown").await?;

        tracing::info!(host = %host, "Successfully subscribed to Sentinel events");

        let mut stream = pubsub.on_message();
        while let Some(msg) = stream.next().await {
            let channel: String = msg.get_channel()?;
            let payload: String = msg.get_payload()?;

            tracing::debug!(
                channel = %channel,
                payload = %payload,
                "Received Sentinel event"
            );

            match channel.as_str() {
                "+switch-master" => {
                    if payload.starts_with(&self.master_name) {
                        tracing::warn!(
                            payload = %payload,
                            "Master switched! Triggering failover handling"
                        );
                        self.on_failover_detected().await;
                    }
                }
                "+odown" => {
                    if payload.contains("master") && payload.contains(&self.master_name) {
                        tracing::warn!(
                            payload = %payload,
                            "Master is objectively down, failover imminent"
                        );
                        self.on_failover_detected().await;
                    }
                }
                "+sdown" => {
                    if payload.contains("master") && payload.contains(&self.master_name) {
                        tracing::warn!(
                            payload = %payload,
                            "Master is subjectively down"
                        );
                    }
                }
                _ => {
                    // Other events (-sdown, -odown) are informational
                }
            }
        }

        Ok(())
    }
}

impl SentinelConnectionManager {
    /// Create a new multiplexed connection to the Redis master
    /// Uses longer timeouts for remote TLS connections
    pub async fn connect(&self) -> std::result::Result<MultiplexedConnection, RedisError> {
        // Configure longer timeouts for TLS connections over network
        let config = AsyncConnectionConfig::new()
            .set_response_timeout(Some(Duration::from_secs(30)))
            .set_connection_timeout(Some(Duration::from_secs(10)));

        // Try with cached master first
        match self.get_master_client().await {
            Ok(client) => {
                match client
                    .get_multiplexed_async_connection_with_config(&config)
                    .await
                {
                    Ok(conn) => return Ok(conn),
                    Err(e) => {
                        tracing::warn!(error = %e, "Connection to cached master failed, invalidating cache");
                        self.on_failover_detected().await;
                    }
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to get master client");
            }
        }

        // Retry after cache invalidation
        let client = self.get_master_client().await?;
        client
            .get_multiplexed_async_connection_with_config(&config)
            .await
    }
}

pub async fn init_dragonfly_redis(
    ca_cert_bytes: Vec<u8>,
    client_cert_bytes: Vec<u8>,
    client_key_bytes: Vec<u8>,
) -> Result<Arc<DragonflyPool>, KVError> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .ok();

    let tls_certs = build_tls_certs(
        ca_cert_bytes.clone(),
        client_cert_bytes.clone(),
        client_key_bytes.clone(),
    );

    let hosts = get_hosts_from_env();

    let conn_addr: Vec<ConnectionAddr> = hosts
        .iter()
        .map(|ip| ConnectionAddr::TcpTls {
            host: ip.clone(),
            port: REDIS_SENTINEL_PORT,
            insecure: false,
            tls_params: None,
        })
        .collect();

    let dragonfly_pass = std::env::var("DRAGONFLY_PASSWORD")
        .expect("DRAGONFLY_PASSWORD environment variable not set");

    let mut builder =
        SentinelClientBuilder::new(conn_addr, SENTINEL_SERVICE_NAME, SentinelServerType::Master)?;

    builder = builder.set_client_to_sentinel_certificates(tls_certs.clone());

    builder = builder.set_client_to_redis_username("default");
    builder = builder.set_client_to_redis_password(dragonfly_pass);
    builder = builder.set_client_to_redis_certificates(tls_certs.clone());
    builder = builder.set_client_to_redis_tls_mode(redis::TlsMode::Secure);

    let sentinel_client = builder.build().expect("Failed to build SentinelClient");
    let conn_man =
        SentinelConnectionManager::new(sentinel_client, SENTINEL_SERVICE_NAME.to_string())?;

    let conn_man_arc = Arc::new(conn_man);

    // Start failover listener
    let conn_man_for_listener = conn_man_arc.clone();
    let tls_certs_for_listener = tls_certs.clone();
    tokio::spawn(async move {
        conn_man_for_listener
            .start_failover_listener(tls_certs_for_listener)
            .await;
    });

    let pool = DragonflyPool::new(conn_man_arc);

    tracing::info!("Dragonfly connection pool initialized");

    Ok(pool)
}

pub async fn init_dragonfly_redis_for_test() -> Result<Arc<DragonflyPool>, KVError> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .ok();

    let ca_bytes = get_ca_cert_pem().expect("Failed to read CA cert");
    let cert_bytes = get_client_cert_pem().expect("Failed to read client cert");
    let key_bytes = get_client_key_pem().expect("Failed to read client key");

    let tls_certs = build_tls_certs(ca_bytes.clone(), cert_bytes.clone(), key_bytes.clone());

    let hosts = get_hosts_from_env();

    let conn_addr: Vec<ConnectionAddr> = hosts
        .iter()
        .map(|ip| ConnectionAddr::TcpTls {
            host: ip.clone(),
            port: REDIS_SENTINEL_PORT,
            insecure: false,
            tls_params: None,
        })
        .collect();

    let dragonfly_pass = std::env::var("DRAGONFLY_PASSWORD")
        .expect("DRAGONFLY_PASSWORD environment variable not set");

    let mut builder =
        SentinelClientBuilder::new(conn_addr, SENTINEL_SERVICE_NAME, SentinelServerType::Master)?;

    builder = builder.set_client_to_sentinel_certificates(tls_certs.clone());
    builder = builder.set_client_to_redis_username("default");
    builder = builder.set_client_to_redis_password(dragonfly_pass);
    builder = builder.set_client_to_redis_certificates(tls_certs.clone());
    builder = builder.set_client_to_redis_tls_mode(redis::TlsMode::Secure);

    let sentinel_client = builder.build().expect("Failed to build SentinelClient");
    let conn_man =
        SentinelConnectionManager::new(sentinel_client, SENTINEL_SERVICE_NAME.to_string())?;

    let conn_man_arc = Arc::new(conn_man);

    // Start failover listener
    let conn_man_for_listener = conn_man_arc.clone();
    let tls_certs_for_listener = tls_certs.clone();
    tokio::spawn(async move {
        conn_man_for_listener
            .start_failover_listener(tls_certs_for_listener)
            .await;
    });

    let pool = DragonflyPool::new(conn_man_arc);

    Ok(pool)
}

#[derive(Clone)]
pub struct DragonflyKV(Arc<DragonflyPool>);
impl DragonflyKV {
    pub async fn new() -> Result<Self, anyhow::Error> {
        let ca_cert_bytes = get_ca_cert_pem().expect("Failed to read CA cert");
        let client_cert_bytes = get_client_cert_pem().expect("Failed to read client cert");
        let client_key_bytes = get_client_key_pem().expect("Failed to read client key");

        let dragonfly_pool =
            init_dragonfly_redis(ca_cert_bytes, client_cert_bytes, client_key_bytes).await?;
        Ok(Self(dragonfly_pool))
    }
}

const AUTH_FIELD: &str = "auth";

impl KVStore for DragonflyKV {
    async fn read(&self, key: String) -> Result<Option<String>, KVError> {
        let value: Option<String> = self
            .0
            .execute_with_retry(|mut conn| {
                let key = key.clone();
                async move { conn.hget(key, AUTH_FIELD).await }
            })
            .await?;
        Ok(value)
    }

    async fn write(&self, key: String, value: String) -> Result<(), KVError> {
        let result: () = self
            .0
            .execute_with_retry(|mut conn| {
                let key = key.clone();
                let value = value.clone();
                async move { conn.hset::<_, _, _, ()>(key, AUTH_FIELD, value).await }
            })
            .await?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use redis::AsyncCommands;

    #[tokio::test]
    async fn test_dragonfly_connection() {
        let pool = init_dragonfly_redis_for_test()
            .await
            .expect("Failed to init dragonfly redis pool");

        // Get a connection from pool
        let mut conn = pool.get().await.expect("Failed to get Redis connection");

        // Write
        let _: () = conn.set("test:hello", "hi").await.expect("SET failed");

        // Read
        let result: Option<String> = conn.get("test:hello").await.expect("GET failed");
        assert_eq!(result.as_deref(), Some("hi"), "Stored value should match");

        // Cleanup
        let _: () = conn.del("test:hello").await.expect("DEL failed");

        println!("dragonfly cluster connection test passed!");
    }

    #[tokio::test]
    async fn test_dragonfly_validated_connection() {
        let pool = init_dragonfly_redis_for_test()
            .await
            .expect("Failed to init dragonfly redis pool");

        // Get a validated connection
        let mut conn = pool
            .get_validated()
            .await
            .expect("Failed to get validated connection");

        // Write and read
        let _: () = conn
            .set("test:validated", "works")
            .await
            .expect("SET failed");
        let result: Option<String> = conn.get("test:validated").await.expect("GET failed");
        assert_eq!(result.as_deref(), Some("works"));

        // Cleanup
        let _: () = conn.del("test:validated").await.expect("DEL failed");
    }

    #[tokio::test]
    async fn test_dragonfly_pipeline() {
        let pool = init_dragonfly_redis_for_test()
            .await
            .expect("Failed to init dragonfly redis pool");

        let mut conn = pool.get().await.expect("Failed to get connection");

        // Build and execute pipeline
        let mut pipe = redis::pipe();
        for i in 0..10 {
            pipe.set(format!("test:pipe:{}", i), format!("value_{}", i))
                .ignore();
        }
        let _: () = pipe.query_async(&mut conn).await.expect("Pipeline failed");

        // Verify
        let val: Option<String> = conn.get("test:pipe:5").await.expect("GET failed");
        assert_eq!(val.as_deref(), Some("value_5"));

        // Cleanup
        for i in 0..10 {
            let _: () = conn.del(format!("test:pipe:{}", i)).await.unwrap();
        }
    }
}
