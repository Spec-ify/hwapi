mod cpu;
mod handlers;

use axum::http::HeaderValue;
use axum::routing::post;
use axum::{routing::get, Router};
use chrono::Local;
use clap::builder::TypedValueParser;
use clap::{Parser, ValueEnum};
use colored::*;
use cpu::CpuCache;
use handlers::*;
use http::{header, Method};
use log::info;
use log::{Level, LevelFilter, Metadata, Record};
use parsing::pcie::PcieCache;
use parsing::usb::UsbCache;
use std::env;
use tower_http::cors::CorsLayer;

/// Because the error that nom uses is rather lengthy and unintuitive, it's defined here
/// to simplify handling
// pub type NomError<'a> = nom::Err<nom::error::Error<&'a str>>;
/// https://docs.rs/log/latest/log/#implementing-a-logger
struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, _: &Metadata) -> bool {
        // this is configured by calling log::set_max_level, and so this logging implementation logs all kinds of levels
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let level = match record.level() {
                Level::Info => format!("{}", record.level()).bold().blue(),
                Level::Warn => format!("{}", record.level()).bold().yellow(),
                Level::Error => format!("{}", record.level()).bold().red(),
                Level::Debug => format!("{}", record.level()).bold().green(),
                Level::Trace => format!("{}", record.level()).bold().cyan(),
            };
            println!(
                "({})[{}] {}",
                Local::now().to_rfc2822(),
                level,
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

#[derive(ValueEnum, Clone)]
enum LoggingLevel {
    Silent,
    Error,
    Warning,
    Info,
    Debug,
    Trace,
}

#[derive(Parser)]
struct Args {
    /// Set the port to listen on
    #[arg(short = 'p', long = "port", default_value_t = String::from("3000"))]
    port: String,
    /// Level of logging verbosity
    #[arg(short = 'v',
        long = "verbosity",
        default_value_t = LevelFilter::Info,
        value_parser = clap::builder::PossibleValuesParser::new(["TRACE", "DEBUG", "INFO", "WARN", "ERROR", "OFF"])
            .map(|s| s.to_lowercase().parse::<LevelFilter>().unwrap())
        )]
    logging_level: LevelFilter,
}

static LOGGER: SimpleLogger = SimpleLogger;

#[derive(Clone)]
struct AppState {
    pub cpu_cache: CpuCache,
    pub usb_cache: UsbCache,
    pub pcie_cache: PcieCache,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // initialize logging
    let cli_args = Args::parse();
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(cli_args.logging_level))
        .unwrap();
    info!("Application started");
    // parse command line arguments
    // create a new http router and register respective routes and handlers
    let app = Router::new()
        .route("/api/cpus/", get(get_cpu_handler))
        .route("/api/usbs/", get(get_usb_handler))
        .route("/api/usbs/", post(post_usbs_handler))
        .route("/api/pcie/", get(get_pcie_handler))
        .route("/api/pcie/", post(post_pcie_handler))
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST])
                .allow_headers([header::ACCEPT, header::CONTENT_TYPE])
                .allow_origin("*".parse::<HeaderValue>().unwrap()),
        )
        .with_state(AppState {
            cpu_cache: CpuCache::new(),
            usb_cache: UsbCache::new(),
            pcie_cache: PcieCache::new(),
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
