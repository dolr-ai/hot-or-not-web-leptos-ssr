pub mod make_span;
pub mod tracing;
pub mod utils;
use opentelemetry::{
    global,
    trace::{noop::NoopTextMapPropagator, SpanId, TraceId, TracerProvider},
};
use opentelemetry_otlp::{LogExporter, SpanExporter, WithExportConfig};
use opentelemetry_sdk::{
    logs::LoggerProvider,
    metrics::SdkMeterProvider,
    propagation::TraceContextPropagator,
    trace::{IdGenerator, TracerProvider as SdkTracerProvider},
    Resource,
};
use serde::{Deserialize, Serialize};
pub use tracing_subscriber::util::TryInitError;
use tracing_subscriber::{
    filter::ParseError, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer,
};
use utils::default_true;
use uuid::Uuid;

use once_cell::sync::OnceCell;
use tracing_appender::non_blocking::WorkerGuard;
// Static to hold the guard and keep it alive for the lifetime of the app
static FILE_GUARD: OnceCell<WorkerGuard> = OnceCell::new();

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct Config {
    /// Logging and tracing level in the env logger format.
    #[serde(default = "default_level")]
    pub level: String,
    #[serde(default = "default_service_name")]
    pub service_name: String,
    #[serde(default)]
    pub exporter: Exporter,
    #[serde(default = "default_otlp_endpoint")]
    pub otlp_endpoint: String,
    #[serde(default = "default_true")]
    pub propagate: bool,
    #[serde(default = "default_file_path")]
    /// The path to the file to write logs to.
    pub file_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            level: default_level(),
            service_name: default_service_name(),
            exporter: Exporter::default(),
            otlp_endpoint: default_otlp_endpoint(),
            propagate: default_true(),
            file_path: default_file_path(),
        }
    }
}

#[derive(Default, Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum Exporter {
    Stdout,
    Otlp,
    File,
    #[default]
    Both,
    /// File + Console logging (no OTLP)
    FileAndStdout,
    /// File + OTLP + Console logging (all three)
    All,
    /// Only export traces via OTLP (for Jaeger compatibility)
    OtlpTracesOnly,
}

fn default_service_name() -> String {
    "estate_fe".to_string()
}
fn default_file_path() -> String {
    "logs/telemetry.log".to_string()
}

fn default_level() -> String {
    "info,estate_fe=trace".to_string()
}

fn default_otlp_endpoint() -> String {
    "http://localhost:4317/v1/metrics".to_string()
}

#[derive(Debug, thiserror::Error)]
pub enum TelemetryError {
    #[error("Log exporter build error: {0}")]
    LogExporterBuild(opentelemetry_otlp::Error),
    #[error("Trace exporter build error: {0}")]
    TraceExporterBuild(opentelemetry_otlp::Error),
    #[error("Metric exporter build error: {0}")]
    MetricExporterBuild(opentelemetry_otlp::Error),
    #[error("Invalid log directive: {0}")]
    InvalidLogDirective(#[from] ParseError),
    #[error("Subscriber error: {0}")]
    Subscriber(#[from] TryInitError),
    #[error("Otel http metrics error")]
    OtelHttpMetrics,
    #[error("File IO error: {0}")]
    FileIO(#[from] std::io::Error),
}

fn resource(config: &Config) -> Resource {
    use opentelemetry::KeyValue;
    Resource::new(vec![KeyValue::new(
        "service.name",
        config.service_name.clone(),
    )])
}

/// Initialize telemetry with the given config.
///
/// # Notes
/// - The reason the `TracerProvider` is not optional is because without it we
///   don't generate trace ids, which is useful to have when
///   debugging/developing.
///
/// # Errors
/// If any of the configuration is invalid.
pub fn init_telemetry(
    config: &Config,
) -> Result<
    (
        Option<LoggerProvider>,
        SdkTracerProvider,
        Option<SdkMeterProvider>,
    ),
    TelemetryError,
> {
    let resource = resource(config);

    if config.propagate {
        global::set_text_map_propagator(TraceContextPropagator::new());
    } else {
        global::set_text_map_propagator(NoopTextMapPropagator::new());
    }

    match config.exporter {
        Exporter::Stdout => {
            let tracer_provider = init_stdout(&resource, config)?;
            Ok((None, tracer_provider, None))
        }
        Exporter::Otlp => {
            let (logger_provider, tracer_provider, metrics_provider) = init_otlp(config)?;
            Ok((
                Some(logger_provider),
                tracer_provider,
                Some(metrics_provider),
            ))
        }
        Exporter::Both => {
            let (logger_provider, tracer_provider, metrics_provider) =
                init_otlp_with_stdout(config)?;
            Ok((
                Some(logger_provider),
                tracer_provider,
                Some(metrics_provider),
            ))
        }
        Exporter::File => {
            let tracer_provider = init_file(config)?;
            Ok((None, tracer_provider, None))
        }
        Exporter::FileAndStdout => {
            let tracer_provider = init_file_with_stdout(config)?;
            Ok((None, tracer_provider, None))
        }
        Exporter::All => {
            let (logger_provider, tracer_provider, metrics_provider) = init_all(config)?;
            Ok((
                Some(logger_provider),
                tracer_provider,
                Some(metrics_provider),
            ))
        }
        Exporter::OtlpTracesOnly => {
            let tracer_provider = init_otlp_traces_only(config)?;
            Ok((None, tracer_provider, None))
        }
    }
}

fn init_file(config: &Config) -> Result<SdkTracerProvider, TelemetryError> {
    let resource = resource(config);

    // Extract directory from file_path
    let file_path = std::path::Path::new(&config.file_path);
    if let Some(dir) = file_path.parent() {
        std::fs::create_dir_all(dir)?;
    }

    let file_appender = tracing_appender::rolling::daily(
        file_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new(".")),
        file_path
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("telemetry.log")),
    );

    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);
    let _ = FILE_GUARD.set(guard);

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(file_writer)
        .with_ansi(false)
        .with_target(true)
        .with_filter(env_filter(config)?);

    let registry = tracing_subscriber::registry().with(file_layer);

    let tracer_provider =
        tracer_provider(config, resource.clone()).map_err(TelemetryError::TraceExporterBuild)?;
    opentelemetry::global::set_tracer_provider(tracer_provider.clone());

    registry.try_init()?;
    log_panics::init();

    Ok(tracer_provider)
}

fn init_file_with_stdout(config: &Config) -> Result<SdkTracerProvider, TelemetryError> {
    let resource = resource(config);

    // Extract directory from file_path
    let file_path = std::path::Path::new(&config.file_path);
    if let Some(dir) = file_path.parent() {
        std::fs::create_dir_all(dir)?;
    }

    let file_appender = tracing_appender::rolling::daily(
        file_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new(".")),
        file_path
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("telemetry.log")),
    );

    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);
    let _ = FILE_GUARD.set(guard);

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(file_writer)
        .with_ansi(false)
        .with_target(true)
        .with_filter(env_filter(config)?);

    let stdout_layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_file(true)
        .with_line_number(true)
        .with_filter(env_filter(config)?);

    let registry = tracing_subscriber::registry()
        .with(file_layer)
        .with(stdout_layer);

    let tracer_provider =
        tracer_provider(config, resource.clone()).map_err(TelemetryError::TraceExporterBuild)?;
    let tracer = tracer_provider.tracer(config.service_name.clone());
    let filter = env_filter(config)?;
    let tracing_layer = tracing_opentelemetry::layer()
        .with_tracer(tracer)
        .with_filter(filter);

    registry.with(tracing_layer).try_init()?;
    opentelemetry::global::set_tracer_provider(tracer_provider.clone());

    log_panics::init();

    Ok(tracer_provider)
}

fn init_all(
    config: &Config,
) -> Result<(LoggerProvider, SdkTracerProvider, SdkMeterProvider), TelemetryError> {
    let resource = resource(config);

    // Extract directory from file_path
    let file_path = std::path::Path::new(&config.file_path);
    if let Some(dir) = file_path.parent() {
        std::fs::create_dir_all(dir)?;
    }

    let file_appender = tracing_appender::rolling::daily(
        file_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new(".")),
        file_path
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("telemetry.log")),
    );

    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);
    let _ = FILE_GUARD.set(guard);

    // logging
    let logger_provider =
        logger_provider(config, resource.clone()).map_err(TelemetryError::LogExporterBuild)?;
    let otel_layer =
        opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge::new(&logger_provider);
    let filter = env_filter(config)?;
    let otel_layer = otel_layer.with_filter(filter);

    // tracing
    let tracer_provider =
        tracer_provider(config, resource.clone()).map_err(TelemetryError::TraceExporterBuild)?;
    let tracer = tracer_provider.tracer(config.service_name.clone());
    let filter = env_filter(config)?;
    let tracing_layer = tracing_opentelemetry::layer()
        .with_tracer(tracer)
        .with_filter(filter);

    // file layer
    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(file_writer)
        .with_ansi(false)
        .with_target(true)
        .with_filter(env_filter(config)?);

    // stdout layer
    let stdout_layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_file(true)
        .with_line_number(true)
        .with_filter(env_filter(config)?);

    tracing_subscriber::registry()
        .with(file_layer)
        .with(stdout_layer)
        .with(tracing_layer)
        .with(otel_layer)
        .try_init()?;

    // metrics
    let metrics_provider =
        metrics_provider(config, resource.clone()).map_err(TelemetryError::MetricExporterBuild)?;

    global::set_meter_provider(metrics_provider.clone());
    global::set_tracer_provider(tracer_provider.clone());

    log_panics::init();

    Ok((logger_provider, tracer_provider, metrics_provider))
}

fn init_otlp(
    config: &Config,
) -> Result<(LoggerProvider, SdkTracerProvider, SdkMeterProvider), TelemetryError> {
    let resource = resource(config);
    // logging
    let logger_provider =
        logger_provider(config, resource.clone()).map_err(TelemetryError::LogExporterBuild)?;
    let otel_layer =
        opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge::new(&logger_provider);
    let filter = env_filter(config)?;
    let otel_layer = otel_layer.with_filter(filter);

    // tracing
    let tracer_provider =
        tracer_provider(config, resource.clone()).map_err(TelemetryError::TraceExporterBuild)?;
    let tracer = tracer_provider.tracer(config.service_name.clone());
    let filter = env_filter(config)?;
    let tracing_layer = tracing_opentelemetry::layer()
        .with_tracer(tracer)
        .with_filter(filter);

    tracing_subscriber::registry()
        .with(tracing_layer)
        .with(otel_layer)
        .try_init()?;

    // metrics
    let metrics_provider =
        metrics_provider(config, resource.clone()).map_err(TelemetryError::MetricExporterBuild)?;

    global::set_meter_provider(metrics_provider.clone());
    global::set_tracer_provider(tracer_provider.clone());

    log_panics::init();

    Ok((logger_provider, tracer_provider, metrics_provider))
}

fn init_otlp_with_stdout(
    config: &Config,
) -> Result<(LoggerProvider, SdkTracerProvider, SdkMeterProvider), TelemetryError> {
    let resource = resource(config);
    let logger_provider =
        logger_provider(config, resource.clone()).map_err(TelemetryError::LogExporterBuild)?;
    let otel_layer =
        opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge::new(&logger_provider);
    let filter = env_filter(config)?;
    let otel_layer = otel_layer.with_filter(filter);

    let tracer_provider =
        tracer_provider(config, resource.clone()).map_err(TelemetryError::TraceExporterBuild)?;
    let tracer = tracer_provider.tracer(config.service_name.clone());
    let filter = env_filter(config)?;
    let tracing_layer = tracing_opentelemetry::layer()
        .with_tracer(tracer)
        .with_filter(filter);

    let logger_layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_file(true)
        .with_line_number(true)
        .with_filter(env_filter(config)?);

    tracing_subscriber::registry()
        .with(logger_layer)
        .with(tracing_layer)
        .with(otel_layer)
        .try_init()?;

    // metrics
    let metrics_provider =
        metrics_provider(config, resource.clone()).map_err(TelemetryError::MetricExporterBuild)?;

    global::set_meter_provider(metrics_provider.clone());
    global::set_tracer_provider(tracer_provider.clone());
    log_panics::init();

    Ok((logger_provider, tracer_provider, metrics_provider))
}

fn init_otlp_traces_only(config: &Config) -> Result<SdkTracerProvider, TelemetryError> {
    let resource = resource(config);

    // Only set up console logging (no OTLP logs)
    let fmt_layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_file(true)
        .with_line_number(true)
        .with_filter(env_filter(config)?);

    // Set up OTLP traces
    use opentelemetry_otlp::new_exporter;
    let exporter = SpanExporter::new(
        new_exporter()
            .tonic()
            .with_endpoint(config.otlp_endpoint.clone())
            .build_span_exporter()
            .map_err(|e| {
                TelemetryError::TraceExporterBuild(opentelemetry_otlp::Error::from(
                    Box::new(e) as Box<dyn std::error::Error + Send + Sync>
                ))
            })?,
    );

    use opentelemetry_sdk::trace::Config;
    let tracer_provider = SdkTracerProvider::builder()
        .with_config(
            Config::default()
                .with_resource(resource.clone())
                .with_id_generator(UuidGenerator)
                .with_max_events_per_span(64)
                .with_max_attributes_per_span(16),
        )
        .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
        .build();

    let tracer = tracer_provider.tracer(config.service_name.clone());
    let filter = env_filter(config)?;
    let tracing_layer = tracing_opentelemetry::layer()
        .with_tracer(tracer)
        .with_filter(filter);

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(tracing_layer)
        .try_init()?;

    opentelemetry::global::set_tracer_provider(tracer_provider.clone());
    log_panics::init();

    Ok(tracer_provider)
}

fn init_stdout(resource: &Resource, config: &Config) -> Result<SdkTracerProvider, TelemetryError> {
    // logging
    let fmt_layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_file(true)
        .with_line_number(true)
        .with_filter(env_filter(config)?);
    let registry = tracing_subscriber::registry().with(fmt_layer);

    // tracing
    let tracer_provider =
        tracer_provider(config, resource.clone()).map_err(TelemetryError::TraceExporterBuild)?;
    let tracer = tracer_provider.tracer(config.service_name.clone());
    let filter = env_filter(config)?;
    let tracing_layer = tracing_opentelemetry::layer()
        .with_tracer(tracer)
        .with_filter(filter);
    registry.with(tracing_layer).try_init()?;
    opentelemetry::global::set_tracer_provider(tracer_provider.clone());

    log_panics::init();

    Ok(tracer_provider)
}

fn env_filter(config: &Config) -> Result<EnvFilter, TelemetryError> {
    // we purposely avoid the EnvFilter::new API so we can catch invalid
    // directives
    let filter = EnvFilter::new(config.level.clone())
        // https://github.com/open-telemetry/opentelemetry-rust/issues/2877
        .add_directive("hyper=off".parse()?)
        .add_directive("tonic=off".parse()?)
        .add_directive("h2=off".parse()?)
        .add_directive("opentelemetry_sdk=off".parse()?)
        .add_directive("reqwest=off".parse()?);
    Ok(filter)
}

fn tracer_provider(
    config: &Config,
    resource: Resource,
) -> Result<SdkTracerProvider, opentelemetry_otlp::Error> {
    match &config.exporter {
        Exporter::Stdout | Exporter::File | Exporter::FileAndStdout => {
            use opentelemetry_sdk::trace::Config;
            Ok(SdkTracerProvider::builder()
                .with_config(
                    Config::default()
                        .with_resource(resource)
                        .with_id_generator(UuidGenerator)
                        .with_max_events_per_span(64)
                        .with_max_attributes_per_span(16),
                )
                .build())
        }
        Exporter::Otlp | Exporter::Both | Exporter::All | Exporter::OtlpTracesOnly => {
            use opentelemetry_otlp::new_exporter;
            let exporter = SpanExporter::new(
                new_exporter()
                    .tonic()
                    .with_endpoint(config.otlp_endpoint.clone())
                    .build_span_exporter()
                    .map_err(|e| {
                        opentelemetry_otlp::Error::from(
                            Box::new(e) as Box<dyn std::error::Error + Send + Sync>
                        )
                    })?,
            );
            use opentelemetry_sdk::trace::Config;
            let provider = SdkTracerProvider::builder()
                .with_config(
                    Config::default()
                        .with_resource(resource)
                        .with_id_generator(UuidGenerator)
                        .with_max_events_per_span(64)
                        .with_max_attributes_per_span(16),
                )
                .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
                .build();
            Ok(provider)
        }
    }
}

fn logger_provider(
    config: &Config,
    resource: Resource,
) -> Result<LoggerProvider, opentelemetry_otlp::Error> {
    use opentelemetry_otlp::new_exporter;
    let exporter = LogExporter::new(
        new_exporter()
            .tonic()
            .with_endpoint(config.otlp_endpoint.clone())
            .build_log_exporter()
            .map_err(|e| {
                opentelemetry_otlp::Error::from(
                    Box::new(e) as Box<dyn std::error::Error + Send + Sync>
                )
            })?,
    );
    Ok(LoggerProvider::builder()
        .with_resource(resource)
        .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
        .build())
}

fn metrics_provider(
    config: &Config,
    resource: Resource,
) -> Result<SdkMeterProvider, opentelemetry_otlp::Error> {
    use opentelemetry_otlp::new_exporter;
    use opentelemetry_sdk::metrics::reader::{
        DefaultAggregationSelector, DefaultTemporalitySelector,
    };

    let exporter = new_exporter()
        .tonic()
        .with_endpoint(config.otlp_endpoint.clone())
        .build_metrics_exporter(
            Box::new(DefaultAggregationSelector::new()),
            Box::new(DefaultTemporalitySelector::new()),
        )
        .map_err(|e| {
            opentelemetry_otlp::Error::from(Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        })?;

    use std::time::Duration;
    let reader = opentelemetry_sdk::metrics::PeriodicReader::builder(
        exporter,
        opentelemetry_sdk::runtime::Tokio,
    )
    .with_interval(Duration::from_secs(30))
    .build();
    Ok(SdkMeterProvider::builder()
        .with_reader(reader)
        .with_resource(resource)
        .build())
}

#[derive(Debug)]
pub struct UuidGenerator;

impl IdGenerator for UuidGenerator {
    fn new_trace_id(&self) -> TraceId {
        TraceId::from(Uuid::new_v4().as_u128())
    }

    fn new_span_id(&self) -> SpanId {
        SpanId::from(Uuid::new_v4().as_u64_pair().0)
    }
}
