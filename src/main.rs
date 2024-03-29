mod cpu;
mod pcie;
mod usb;

use axum::extract::Query;
use axum::http::{HeaderValue, StatusCode};
use axum::routing::post;
use axum::{extract::State, routing::get, Json, Router};
use chrono::Local;
use clap::Parser;
use colored::*;
use cpu::{Cpu, CpuCache};
use log::{error, info, warn};
use log::{Level, LevelFilter, Metadata, Record};
use pcie::PcieCache;
use serde::{Deserialize, Serialize};
use std::env;
use tower_http::cors::CorsLayer;
use usb::UsbCache;
use http::{Method,header};

/// Because the error that nom uses is rather lengthy and unintuitive, it's defined here
/// to simplify handling
pub type NomError<'a> = nom::Err<nom::error::Error<&'a str>>;
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
struct AppState<'a> {
    pub cpu_cache: CpuCache<'a>,
    pub usb_cache: UsbCache,
    pub pcie_cache: PcieCache,
}

#[derive(Debug, Deserialize, Serialize)]
struct UsbQuery {
    pub identifier: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct UsbResponse {
    pub vendor: Option<String>,
    pub device: Option<String>,
}

/// This handler accepts a `GET` request to `/api/usbs/?identifier`.
/// It relies on a globally shared [AppState] to re-use the usb cache.
async fn get_usb_handler<'a>(
    State(state): State<AppState<'a>>,
    Query(query): Query<UsbQuery>,
) -> Result<Json<UsbResponse>, StatusCode> {
    // TODO: update docs
    let results = state.usb_cache.find(&query.identifier);
    match results {
        Ok(r) => Ok(Json(UsbResponse {
            vendor: r.0.map(|v| v.name),
            device: r.1.map(|d| d.name),
        })),
        Err(e) => {
            error!("usb handler error: {:?} caused by query: {:?}", e, query);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct GetPcieQuery {
    identifier: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct PcieResponse {
    pub vendor: Option<String>,
    pub device: Option<String>,
    pub subsystem: Option<String>,
}

/// This handler accepts a `GET` request to `/api/pcie/?identifier`.
/// It relies on a globally shared [AppState] to re-use the pcie cache
async fn get_pcie_handler(
    State(state): State<AppState<'_>>,
    Query(query): Query<GetPcieQuery>,
) -> Result<Json<PcieResponse>, StatusCode> {
    let results = state.pcie_cache.find(&query.identifier);
    match results {
        Ok(r) => Ok(Json(PcieResponse {
            vendor: r.0.map(|v| v.name),
            device: r.1.map(|d| d.name),
            subsystem: r.2.map(|s| s.name),
        })),
        Err(e) => {
            error!("pcie handler error: {:?} caused by query: {:?}", e, query);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// This handler accepts a `POST` request to `/api/pcie/`, with a body containing a serialized array of strings.
/// It relies on a globally shared [AppState] to re-use the pcie cache, and is largely identical to [get_pcie_handler], but
/// is intended for batching
async fn post_pcie_handler(
    State(state): State<AppState<'_>>,
    Json(query): Json<Vec<String>>,
) -> Result<Json<Vec<Option<PcieResponse>>>, StatusCode> {
    let mut response: Vec<Option<PcieResponse>> = Vec::with_capacity(16);
    for entry in query {
        match state.pcie_cache.find(&entry) {
            Ok(r) => response.push(Some(PcieResponse {
                vendor: r.0.map(|v| v.name),
                device: r.1.map(|d| d.name),
                subsystem: r.2.map(|s| s.name),
            })),
            Err(e) => {
                warn!("post pcie handler error: when processing the device identifier {:?}, an error was returned: {:?}", entry, e);
                response.push(None);
            }
        }
    }
    Ok(Json(response))
}

/// This handler accepts a `POST` request to `/api/usbs/`, with a body containing a serialized array of usb device identifier strings.
/// It relies on a globally shared [AppState] to re-use the pcie cache, and is largely identical to [get_usb_handler], but
/// is intended for batching
async fn post_usbs_handler(
    State(state): State<AppState<'_>>,
    Json(query): Json<Vec<String>>,
) -> Result<Json<Vec<Option<UsbResponse>>>, StatusCode> {
    let mut response: Vec<Option<UsbResponse>> = Vec::with_capacity(16);
    for entry in query {
        match state.usb_cache.find(&entry) {
            Ok(r) => response.push(Some(UsbResponse {
                vendor: r.0.map(|v| v.name),
                device: r.1.map(|d| d.name),
            })),
            Err(e) => {
                warn!("post usb handler error: when processing the device identifier {:?}, an error was returned: {:?}", entry, e);
                response.push(None);
            }
        }
    }
    Ok(Json(response))
}

#[derive(Debug, Deserialize, Serialize)]
struct CpuQuery {
    pub name: String,
}

/// This handler accepts a `GET` request to `/api/cpus/?name=[CPU_NAME]`.
/// It relies on a globally shared [AppState] to re-use the cpu cache, and responds to the request with a serialized [Cpu].
/// It will always attempt to find a cpu, and should always return a cpu. The correctness of the return value is not guaranteed.
async fn get_cpu_handler<'a>(
    State(mut state): State<AppState<'a>>,
    Query(query): Query<CpuQuery>,
) -> Result<Json<Cpu<String>>, StatusCode> {
    match state.cpu_cache.find(&query.name) {
        Ok(c) => Ok(Json(Cpu {
            name: c.name.to_string(),
            attributes: c
                .attributes
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        })),
        Err(e) => {
            error!("cpu handler error {:?} caused by query {:?}", e, query);
            Err(StatusCode::NOT_FOUND)
        }
    }
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
        .route("/api/usbs/", get(get_usb_handler))
        .route("/api/usbs/", post(post_usbs_handler))
        .route("/api/pcie/", get(get_pcie_handler))
        .route("/api/pcie/", post(post_pcie_handler))
        .layer(CorsLayer::new().allow_methods([Method::GET, Method::POST]).allow_headers([header::ACCEPT, header::CONTENT_TYPE]).allow_origin("*".parse::<HeaderValue>().unwrap()))
        .with_state(AppState {
            cpu_cache: CpuCache::new(),
            usb_cache: UsbCache::new(),
            pcie_cache: PcieCache::new(),
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
