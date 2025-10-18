// use clap::Subcommand;
use opentelemetry::{global, trace::Span};

use opentelemetry::{trace::Tracer, KeyValue};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    logs::SdkLoggerProvider, propagation::TraceContextPropagator, trace::SdkTracerProvider,
};
use opentelemetry_stdout::{LogExporter, SpanExporter};
use tracing::{trace, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub struct OtelConfig {
    pub exporter: Option<String>,
    pub endpoint: Option<String>,
    pub protocol: Option<String>,
}

pub fn init_tracer(otel_config: OtelConfig) -> SdkTracerProvider {
    global::set_text_map_propagator(TraceContextPropagator::new());
    let otel_service_config = {
        // Here you can set up resource attributes like service name, version, etc.
        opentelemetry_sdk::Resource::builder()
            .with_attributes(vec![KeyValue::new("service.name", "pact-cli")])
            .build()
    };
    let provider = match otel_config.exporter.as_deref() {
        Some("otlp") => {
            let endpoint = otel_config
                .endpoint
                .unwrap_or_else(|| "http://localhost:4318".to_string());
            let protocol = otel_config.protocol.unwrap_or_else(|| "http".to_string());

            let otlp_exporter = {
                trace!(
                    "Initializing OTLP exporter with endpoint: {} and protocol: {}",
                    endpoint,
                    protocol
                );
                match protocol.as_str() {
                    "grpc" => opentelemetry_otlp::SpanExporter::builder()
                        .with_tonic()
                        .with_endpoint(endpoint.to_string())
                        .build()
                        .expect("Failed to configure grpc exporter"),
                    _ => opentelemetry_otlp::SpanExporter::builder()
                        .with_http()
                        .with_endpoint(endpoint.to_string() + "/v1/traces")
                        .build()
                        .expect("Failed to configure http exporter"),
                }
            };

            SdkTracerProvider::builder()
                .with_simple_exporter(otlp_exporter)
                .with_resource(otel_service_config)
                .build()
        }
        _ => SdkTracerProvider::builder()
            .with_simple_exporter(SpanExporter::default())
            .with_resource(otel_service_config)
            .build(),
    };

    global::set_tracer_provider(provider.clone());
    provider
}

pub fn init_logs(log_level: Option<Level>) -> Option<SdkLoggerProvider> {
    // Setup logger provider with stdout exporter
    let logger_provider = SdkLoggerProvider::builder()
        .with_simple_exporter(LogExporter::default())
        .build();
    let otel_layer = OpenTelemetryTracingBridge::new(&logger_provider);

    // Instead of .init(), attach to existing tracing subscribers
    if tracing_subscriber::registry()
        .with(otel_layer)
        .with(
            tracing_subscriber::fmt::layer()
                .compact()
                .with_thread_names(true)
                .with_level(true),
        )
        .with({
            if let Some(level) = log_level {
                Some(tracing_subscriber::filter::LevelFilter::from_level(level))
            } else {
                Some(tracing_subscriber::filter::LevelFilter::OFF)
            }
        })
        .try_init()
        .is_ok()
    {
        Some(logger_provider)
    } else {
        // Failed to initialize, likely due to dispatcher already set
        None
    }
}

pub fn capture_telemetry(args: &[String], exit_code: i32, error_message: Option<&str>) {
    let tracer = global::tracer("pact-cli");
    let mut span = tracer.start("invocation");

    // set the service name and other otlp high level attributes

    if let Some(binary) = args.get(0) {
        span.set_attribute(KeyValue::new("binary", binary.clone()));
    }
    if let Some(command) = args.get(1) {
        span.set_attribute(KeyValue::new("command", command.clone()));
    }
    if let Some(subcommand) = args.get(2) {
        span.set_attribute(KeyValue::new("subcommand", subcommand.clone()));
    }
    if args.len() > 3 {
        span.set_attribute(KeyValue::new("args", format!("{:?}", &args[3..])));
    }
    span.set_attribute(KeyValue::new("exit_code", exit_code as i64));
    if let Some(message) = error_message {
        span.set_attribute(KeyValue::new("error_message", message.to_string()));
    }
    span.end();
}
