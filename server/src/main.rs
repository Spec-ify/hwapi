//! This module contains the code centered around the actual server binary. It relies on handlers from the `handlers` crate, which fetch data from
//! interfaces provided by the `database` crate, which rely on data parsed by the `parsing` crate.

use axum::extract::{MatchedPath, Request};
use axum::http::{HeaderValue, Method, header};
use axum::routing::post;
use axum::{Router, routing::get};
use clap::Parser;
use clap::builder::TypedValueParser;
use databases::bugcheck::BugCheckCache;
use databases::cpu::CpuCache;
use databases::pcie::PcieCache;
use databases::usb::UsbCache;
use handlers::*;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace::SdkTracerProvider;
use std::env;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{Level, info, info_span};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Parser)]
struct Args {
    /// Set the port to listen on
    #[arg(short = 'p', long = "port", default_value_t = String::from("3000"))]
    port: String,
    /// Level of logging verbosity
    #[arg(short = 'v',
        long = "verbosity",
        default_value_t = Level::INFO,
        value_parser = clap::builder::PossibleValuesParser::new(["TRACE", "DEBUG", "INFO", "WARN", "ERROR", "OFF"])
            .map(|s| s.to_lowercase().parse::<Level>().unwrap())
        )]
    logging_level: Level,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // initialize logging
    let cli_args = Args::parse();
    let fmt_layer = tracing_subscriber::fmt::layer();
    let otel_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_protocol(opentelemetry_otlp::Protocol::HttpJson)
        .with_endpoint("https://oltp.spec-ify.com/v1/traces")
        .build()?;
    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(otel_exporter)
        .with_resource(Resource::builder().with_service_name("hwapi").build())
        .build();
    let tracer = provider.tracer("hwapi");
    let otel_layer: tracing_opentelemetry::OpenTelemetryLayer<
        tracing_subscriber::layer::Layered<
            tracing_subscriber::fmt::Layer<tracing_subscriber::Registry>,
            tracing_subscriber::Registry,
        >,
        opentelemetry_sdk::trace::Tracer,
    > = tracing_opentelemetry::layer().with_tracer(tracer);
    let registry = tracing_subscriber::registry().with(fmt_layer);
    if cfg!(debug_assertions) {
        registry.init();
    } else {
        registry.with(otel_layer).init();
    }
    info!("Application started");
    // parse command line arguments
    // create a new http router and register respective routes and handlers
    let app = Router::new()
        .route("/api/hello/", get(|| async { "hi mom!" }))
        .route("/api/cpus/", get(get_cpu_handler))
        .route("/api/usbs/", get(get_usb_handler))
        .route("/api/usbs/", post(post_usbs_handler))
        .route("/api/pcie/", get(get_pcie_handler))
        .route("/api/pcie/", post(post_pcie_handler))
        .route("/api/bugcheck/", get(get_bugcheck_handler))
        .route("/api/bugcheck/", post(post_bugcheck_handler))
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST])
                .allow_headers([header::ACCEPT, header::CONTENT_TYPE])
                .allow_origin("*".parse::<HeaderValue>().unwrap()),
        )
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                // Log the matched route's path (with placeholders not filled in).
                // Use request.uri() or OriginalUri if you want the real path.
                let matched_path = request
                    .extensions()
                    .get::<MatchedPath>()
                    .map(MatchedPath::as_str);
                info_span!(
                    "http_request",
                    method = ?request.method(),
                    path=matched_path,
                    some_other_field = tracing::field::Empty,
                )
            }),
        )
        .with_state(AppState {
            cpu_cache: CpuCache::new(),
            usb_cache: UsbCache::new(),
            pcie_cache: PcieCache::new(),
            bugcheck_cache: BugCheckCache::new(),
        });

    let mut port: String = cli_args.port;
    if let Ok(value) = env::var("HWAPI_PORT") {
        port = value;
    }

    info!("Listening on port {}", port);
    // run the app
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
