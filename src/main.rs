mod cpu;

use axum::extract::Query;
use axum::http::HeaderValue;
use axum::{extract::State, routing::get, Json, Router};
use chrono::Local;
use clap::Parser;
use colored::*;
use cpu::{Cpu, CpuCache};
use log::info;
use log::{Level, LevelFilter, Metadata, Record};
use serde::{Deserialize, Serialize};
use std::env;
use tower_http::cors::CorsLayer;
/// https://docs.rs/log/latest/log/#implementing-a-logger
struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        // determine at what level things will be logged at
        // TODO: make this configurable via environment variable
        metadata.level() <= Level::Info
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

#[derive(Parser)]
struct Args {
    #[arg(short = 'p', long = "port")]
    port: Option<String>,
}

static LOGGER: SimpleLogger = SimpleLogger;

#[derive(Clone)]
struct AppState {
    pub cpu_cache: CpuCache,
}

#[derive(Debug, Deserialize, Serialize)]
struct CpuQuery {
    pub name: String,
}

/// This handler accepts a `GET` request to `/api/cpus/?name=[CPU_NAME]`.
/// It relies on a globally shared [AppState] to re-use the cpu cache, and responds to the request with a serialized [Cpu].
/// It will always attempt to find a cpu, and should always return a cpu. The correctness of the return value is not guaranteed.
async fn get_cpu_handler(
    State(state): State<AppState>,
    Query(query): Query<CpuQuery>,
) -> Json<Cpu> {
    // just to get type annotations working
    let mut state: AppState = state;
    Json(state.cpu_cache.find(&query.name))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // initialize logging
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
        .unwrap();
    let cli_args = Args::parse();
    info!("Application started");
    // parse command line arguments
    // create a new http router and register respective routes and handlers
    let app = Router::new()
        .route("/api/cpus/", get(get_cpu_handler))
        .layer(CorsLayer::new().allow_origin("*".parse::<HeaderValue>().unwrap()))
        .with_state(AppState {
            cpu_cache: CpuCache::new(),
        });

    let mut port: String = String::from("3000");
    if let Ok(value) = env::var("HWAPI_PORT") {
        port = value;
    } else if let Some(value) = cli_args.port {
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
