#![recursion_limit = "256"]
use axum::{
    body::Body as AxumBody,
    extract::{Path, State},
    http::Request,
    response::{IntoResponse, Response},
};
use axum::{routing::get, Router};
use hot_or_not_web_leptos_ssr::fallback::file_and_error_handler;
use sentry_tower::{NewSentryLayer, SentryHttpLayer};
use state::server::AppState;
use tower::ServiceBuilder;
use tracing::instrument;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utils::host::is_host_or_origin_from_preview_domain;

use hot_or_not_web_leptos_ssr::app::shell;
use hot_or_not_web_leptos_ssr::{app::App, init::AppStateBuilder};
use http::{header, HeaderName, Method};
use leptos::logging::log;
use leptos::prelude::*;
use leptos_axum::handle_server_fns_with_context;
use leptos_axum::{generate_route_list, LeptosRoutes};
use tower_http::cors::{AllowOrigin, CorsLayer};

#[instrument(skip(app_state))]
pub async fn server_fn_handler(
    State(app_state): State<AppState>,
    path: Path<String>,
    request: Request<AxumBody>,
) -> impl IntoResponse {
    log!("{:?}", path);

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
            provide_context(app_state.google_oauth_clients.clone());

            #[cfg(feature = "ga4")]
            provide_context(app_state.grpc_offchain_channel.clone());

            #[cfg(feature = "firestore")]
            provide_context(app_state.firestore_db.clone());

            #[cfg(feature = "qstash")]
            provide_context(app_state.qstash.clone());

            provide_context(app_state.grpc_icpump_search_channel.clone());
            provide_context(app_state.grpc_nsfw_channel.clone());
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
            provide_context(app_state.google_oauth_clients.clone());

            #[cfg(feature = "ga4")]
            provide_context(app_state.grpc_offchain_channel.clone());

            #[cfg(feature = "firestore")]
            provide_context(app_state.firestore_db.clone());

            #[cfg(feature = "qstash")]
            provide_context(app_state.qstash.clone());

            provide_context(app_state.grpc_icpump_search_channel.clone());
            provide_context(app_state.grpc_nsfw_channel.clone());
        },
        move || shell(app_state.leptos_options.clone()),
    );
    handler(state, req).await.into_response()
}

async fn main_impl() {
    dotenv::dotenv().ok();

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

    let sentry_tower_layer = ServiceBuilder::new()
        .layer(NewSentryLayer::new_from_top())
        .layer(SentryHttpLayer::with_transaction());

    // build our application with a route
    let app = Router::new()
        .route(
            "/api/*fn_name",
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
        .layer(sentry_tower_layer)
        .with_state(res.app_state);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log::info!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(terminate)
        .await
        .unwrap();
}

fn main() {
    let _guard = sentry::init((
        "https://385626ba180040d470df02ac5ba1c6f4@sentry.yral.com/4",
        sentry::ClientOptions {
            release: sentry::release_name!(),
            debug: true,
            traces_sample_rate: 0.25,
            environment: Some("staging-komal-preview".into()),
            ..Default::default()
        },
    ));

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                // axum logs rejections from built-in extractors with the `axum::rejection`
                // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
                format!(
                    "{}=debug,tower_http=debug,axum::rejection=trace",
                    env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(sentry_tracing::layer())
        .init();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            main_impl().await;
        });
}
