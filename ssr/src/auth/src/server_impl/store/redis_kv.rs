use bb8_redis::RedisConnectionManager;
use redis::{AsyncCommands, RedisError};

use super::{KVError, KVStore};

#[derive(Clone)]
pub struct RedisKV(bb8::Pool<RedisConnectionManager>);

impl RedisKV {
    pub async fn new(redis_url: &str) -> Result<Self, bb8::RunError<RedisError>> {
        let manager = RedisConnectionManager::new(redis_url)?;
        Ok(Self(bb8::Pool::builder().build(manager).await?))
    }
}

const AUTH_FIELD: &str = "auth";

impl KVStore for RedisKV {
    async fn read(&self, key: String) -> Result<Option<String>, KVError> {
        let mut con = self.0.get().await?;
        let value: Option<String> = con.hget(key, AUTH_FIELD).await?;
        Ok(value)
    }

    async fn write(&self, key: String, value: String) -> Result<(), KVError> {
        let mut con = self.0.get().await?;
        con.hset::<_, _, _, ()>(key, AUTH_FIELD, value).await?;
        Ok(())
    }
}
