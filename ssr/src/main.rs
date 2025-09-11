#![recursion_limit = "256"]
use axum::{
    body::Body as AxumBody,
    extract::State,
    http::Request,
    response::{IntoResponse, Response},
};
use axum::{routing::get, Router};
use hot_or_not_web_leptos_ssr::fallback::file_and_error_handler;
use sentry_tower::{NewSentryLayer, SentryHttpLayer};
use state::server::AppState;
use tower::ServiceBuilder;
use tracing::instrument;
use utils::host::is_host_or_origin_from_preview_domain;

use hot_or_not_web_leptos_ssr::app::shell;
use hot_or_not_web_leptos_ssr::{app::App, init::AppStateBuilder};
use http::{header, HeaderName, Method};
use leptos::prelude::*;
use leptos_axum::handle_server_fns_with_context;
use leptos_axum::{generate_route_list, LeptosRoutes};
use tower_http::cors::{AllowOrigin, CorsLayer};

#[instrument(skip(app_state))]
pub async fn server_fn_handler(
    State(app_state): State<AppState>,
    request: Request<AxumBody>,
) -> impl IntoResponse {
    handle_server_fns_with_context(
        move || {
            provide_context(app_state.canisters.clone());
            #[cfg(feature = "backend-admin")]
            provide_context(app_state.admin_canisters.clone());
            #[cfg(feature = "cloudflare")]
            provide_context(app_state.cloudflare.clone());
            provide_context(app_state.kv.clone());
            provide_context(app_state.cookie_key.clone());

            #[cfg(feature = "oauth-ssr")]
            {
                provide_context(app_state.yral_oauth_client.clone());
                provide_context(app_state.yral_auth_migration_key.clone());
            }

            #[cfg(feature = "ga4")]
            provide_context(app_state.grpc_offchain_channel.clone());

            #[cfg(feature = "qstash")]
            provide_context(app_state.qstash.clone());

            #[cfg(feature = "alloydb")]
            {
                provide_context(app_state.alloydb.clone());
                provide_context(app_state.hon_worker_jwt.clone());
            }
            #[cfg(feature = "dolr-airdrop")]
            provide_context(app_state.dolr_airdrop_db.clone());
            #[cfg(feature = "sats-airdrop")]
            {
                provide_context(app_state.sats_airdrop_db.clone());
                provide_context(app_state.hon_worker_jwt.clone());
            }
        },
        request,
    )
    .await
}

#[instrument(skip(state))]
pub async fn leptos_routes_handler(state: State<AppState>, req: Request<AxumBody>) -> Response {
    let State(app_state) = state.clone();
    let handler = leptos_axum::render_route_with_context(
        app_state.routes.clone(),
        move || {
            provide_context(app_state.canisters.clone());
            #[cfg(feature = "backend-admin")]
            provide_context(app_state.admin_canisters.clone());
            #[cfg(feature = "cloudflare")]
            provide_context(app_state.cloudflare.clone());
            provide_context(app_state.kv.clone());
            provide_context(app_state.cookie_key.clone());
            #[cfg(feature = "oauth-ssr")]
            provide_context(app_state.yral_oauth_client.clone());

            #[cfg(feature = "ga4")]
            provide_context(app_state.grpc_offchain_channel.clone());

            #[cfg(feature = "qstash")]
            provide_context(app_state.qstash.clone());

            #[cfg(feature = "alloydb")]
            {
                provide_context(app_state.alloydb.clone());
                provide_context(app_state.hon_worker_jwt.clone());
            }
            #[cfg(feature = "dolr-airdrop")]
            provide_context(app_state.dolr_airdrop_db.clone());
            #[cfg(feature = "sats-airdrop")]
            {
                provide_context(app_state.sats_airdrop_db.clone());
                provide_context(app_state.hon_worker_jwt.clone());
            }
        },
        move || shell(app_state.leptos_options.clone()),
    );
    handler(state, req).await.into_response()
}

async fn main_impl() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    // Initialize telemetry if OTLP_ENDPOINT is configured
    #[cfg(feature = "enable-oltp")]
    let telemetry_handles = setup_telemetry();

    // Setting get_configuration(None) means we'll be using cargo-leptos's env values
    // For deployment these variables are:
    // <https://github.com/leptos-rs/start-axum#executing-a-server-on-a-remote-machine-without-the-toolchain>
    // Alternately a file can be specified such as Some("Cargo.toml")
    // The file would need to be included with the executable when moved to deployment
    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let res = AppStateBuilder::new(leptos_options, routes.clone())
        .build()
        .await;
    let terminate = {
        use tokio::signal;

        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            use tokio::signal;
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        async {
            tokio::select! {
                _ = ctrl_c => {},
                _ = terminate => {},
            }
            log::info!("stopping...");

            #[cfg(feature = "local-bin")]
            std::mem::drop(res.containers);
        }
    };

    // Create HTTP tracing layer with OpenTelemetry semantic conventions
    #[cfg(feature = "enable-oltp")]
    let trace_layer = tower_http::trace::TraceLayer::new_for_http()
        .make_span_with(|request: &axum::extract::Request<_>| {
            let method = request.method();
            let uri = request.uri();
            let route = request
                .extensions()
                .get::<axum::extract::MatchedPath>()
                .map(|path| path.as_str())
                .unwrap_or_else(|| uri.path());

            tracing::info_span!(
                "http_request",
                method = %method,
                uri = %uri,
                route = route,
                // OpenTelemetry semantic conventions
                otel.name = format!("{} {}", method, route),
                otel.kind = "server",
                http.method = %method,
                http.url = %uri,
                http.route = route,
                service.name = "yral_ssr",
            )
        })
        .on_response(
            |response: &axum::response::Response,
             latency: std::time::Duration,
             _span: &tracing::Span| {
                tracing::info!(
                    status_code = response.status().as_u16(),
                    latency_ms = latency.as_millis(),
                    "request completed"
                );
            },
        );

    let sentry_tower_layer = ServiceBuilder::new()
        .layer(NewSentryLayer::new_from_top())
        .layer(SentryHttpLayer::with_transaction());

    // build our application with a route
    let app = Router::new()
        .route(
            "/api/{*fn_name}",
            get(server_fn_handler).post(server_fn_handler),
        )
        .layer(
            CorsLayer::new()
                .allow_credentials(true)
                .allow_headers([
                    header::AUTHORIZATION,
                    header::CONTENT_TYPE,
                    header::ACCEPT,
                    HeaderName::from_static("sentry-trace"),
                    HeaderName::from_static("baggage"),
                ])
                .allow_methods([Method::POST, Method::GET, Method::PUT, Method::OPTIONS])
                .allow_origin(AllowOrigin::predicate(|origin, _| {
                    if let Ok(host) = origin.to_str() {
                        is_host_or_origin_from_preview_domain(host) || host == "yral.com"
                    } else {
                        false
                    }
                })),
        )
        .leptos_routes_with_handler(routes, get(leptos_routes_handler))
        .fallback(file_and_error_handler)
        // .layer(trace_layer)
        .layer(sentry_tower_layer)
        .with_state(res.app_state);

    #[cfg(feature = "enable-oltp")]
    let app = app.layer(trace_layer);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    println!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(terminate)
    .await
    .unwrap();

    // Cleanup telemetry providers if they were initialized
    #[cfg(feature = "enable-oltp")]
    if let Some((logger_provider, tracer_provider, metrics_provider)) = telemetry_handles {
        if let Some(logger_provider) = logger_provider {
            if let Err(e) = logger_provider.shutdown() {
                eprintln!("Error shutting down logger provider: {e}");
            }
        }
        if let Err(e) = tracer_provider.shutdown() {
            eprintln!("Error shutting down tracer provider: {e}");
        }
        if let Some(metrics_provider) = metrics_provider {
            if let Err(e) = metrics_provider.shutdown() {
                eprintln!("Error shutting down metrics provider: {e}");
            }
        }
        tracing::info!("Telemetry providers shut down");
    }
    Ok(())
}

#[cfg(feature = "enable-oltp")]
fn setup_telemetry() -> Option<(
    Option<opentelemetry_sdk::logs::LoggerProvider>,
    opentelemetry_sdk::trace::TracerProvider,
    Option<opentelemetry_sdk::metrics::SdkMeterProvider>,
)> {
    let telemetry_handles = if let Ok(otlp_endpoint) = std::env::var("OTLP_ENDPOINT") {
        // Use OtlpTracesOnly for Jaeger - traces include latency metrics
        // Logs emitted during request handling will appear as span events in Jaeger
        let telemetry_config = telemetry_axum::Config {
            exporter: telemetry_axum::Exporter::OtlpTracesOnly, // Traces with embedded logs to Jaeger
            otlp_endpoint: otlp_endpoint.clone(),
            service_name: "yral_ssr".to_string(),
            level: "info,yral_ssr=debug,tower_http=info,hot_or_not_web_leptos_ssr=debug"
                .to_string(),
            propagate: true, // Enable trace propagation for distributed tracing
            ..Default::default()
        };

        match telemetry_axum::init_telemetry(&telemetry_config) {
            Ok(handles) => {
                tracing::info!(
                    "Telemetry initialized with Jaeger endpoint at {} (traces only, logs to stdout)",
                    otlp_endpoint
                );
                Some(handles)
            }
            Err(e) => {
                eprintln!("Warning: Failed to initialize telemetry with OTLP endpoint {otlp_endpoint}: {e}. Falling back to stdout only.");

                // Fallback to stdout-only logging
                let telemetry_config = telemetry_axum::Config {
                    exporter: telemetry_axum::Exporter::Stdout,
                    service_name: "yral_ssr".to_string(),
                    level: "info,yral_ssr=debug,tower_http=info,hot_or_not_web_leptos_ssr=debug"
                        .to_string(),
                    ..Default::default()
                };

                match telemetry_axum::init_telemetry(&telemetry_config) {
                    Ok(handles) => {
                        tracing::info!(
                            "Telemetry initialized with stdout-only logging (Jaeger unavailable)"
                        );
                        Some(handles)
                    }
                    Err(e) => {
                        eprintln!("Error: Failed to initialize fallback telemetry: {e}");
                        None
                    }
                }
            }
        }
    } else {
        // No OTLP_ENDPOINT configured, use stdout-only logging
        let telemetry_config = telemetry_axum::Config {
            exporter: telemetry_axum::Exporter::Stdout,
            service_name: "yral_ssr".to_string(),
            level: "info,yral_ssr=debug,tower_http=info,hot_or_not_web_leptos_ssr=debug"
                .to_string(),
            ..Default::default()
        };

        match telemetry_axum::init_telemetry(&telemetry_config) {
            Ok(handles) => {
                tracing::info!(
                    "Telemetry initialized with stdout-only logging (no OTLP_ENDPOINT configured)"
                );
                Some(handles)
            }
            Err(e) => {
                eprintln!("Error: Failed to initialize telemetry: {e}");
                None
            }
        }
    };
    telemetry_handles
}

fn main() {
    let _guard = sentry::init((
        "https://385626ba180040d470df02ac5ba1c6f4@sentry.yral.com/4",
        sentry::ClientOptions {
            release: sentry::release_name!(),
            debug: true,
            traces_sample_rate: 0.25,
            ..Default::default()
        },
    ));

    // Note: telemetry_axum will initialize its own tracing subscriber
    // that includes OpenTelemetry exporters for Jaeger

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            if let Err(e) = main_impl().await {
                eprintln!("Server error: {e}");
                std::process::exit(1);
            }
        });
}
