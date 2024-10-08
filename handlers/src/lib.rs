//! This crate contains the Axum handlers used by the server.
use axum::extract::Query;
use axum::http::StatusCode;
use axum::{extract::State, Json};
use databases::bugcheck::BugCheckCache;
use databases::cpu::Cpu;
use databases::{cpu::CpuCache, pcie::PcieCache, usb::UsbCache};
use serde::{Deserialize, Serialize};
use tracing::{error, warn};

#[derive(Clone)]
pub struct AppState {
    pub cpu_cache: CpuCache,
    pub usb_cache: UsbCache,
    pub pcie_cache: PcieCache,
    pub bugcheck_cache: BugCheckCache,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UsbQuery {
    pub identifier: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UsbResponse {
    pub vendor: Option<String>,
    pub device: Option<String>,
}

/// This handler accepts a `GET` request to `/api/usbs/?identifier`.
/// It relies on a globally shared [AppState] to re-use the usb cache.
#[tracing::instrument(name = "single_usb_handler", skip(state))]
pub async fn get_usb_handler(
    State(state): State<AppState>,
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
pub struct GetPcieQuery {
    identifier: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PcieResponse {
    pub vendor: Option<String>,
    pub device: Option<String>,
    pub subsystem: Option<String>,
}

/// This handler accepts a `GET` request to `/api/pcie/?identifier`.
/// It relies on a globally shared [AppState] to re-use the pcie cache
#[tracing::instrument(name = "single_pcie_handler", skip(state))]
pub async fn get_pcie_handler(
    State(state): State<AppState>,
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
#[tracing::instrument(name = "bulk_pcie_handler", skip(state))]
pub async fn post_pcie_handler(
    State(state): State<AppState>,
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
#[tracing::instrument(name = "bulk_usb_handler", skip(state))]
pub async fn post_usbs_handler(
    State(state): State<AppState>,
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
pub struct CpuQuery {
    pub name: String,
}

/// This handler accepts a `GET` request to `/api/cpus/?name=[CPU_NAME]`.
/// It relies on a globally shared [AppState] to re-use the cpu cache, and responds to the request with a serialized [Cpu].
/// It will always attempt to find a cpu, and should always return a cpu. The correctness of the return value is not guaranteed.
#[tracing::instrument(name = "cpu_handler", skip(state))]
pub async fn get_cpu_handler(
    State(mut state): State<AppState>,
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

#[derive(Serialize, Deserialize)]
pub struct GetBugCheckQuery {
    code: u64,
}

#[derive(Deserialize, Serialize)]
pub struct BugCheckResponse {
    code: u64,
    name: String,
    url: String,
}
/// This handler accepts a `GET` request to `/api/bugcheck/?code=[CODE]`
pub async fn get_bugcheck_handler(
    State(state): State<AppState>,
    Query(query): Query<GetBugCheckQuery>,
) -> Result<Json<BugCheckResponse>, StatusCode> {
    if let Some((name, url)) = state.bugcheck_cache.get(query.code) {
        Ok(Json(BugCheckResponse {
            code: query.code,
            name: name.to_string(),
            url: url.to_string(),
        }))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// This handler accepts a `POST` request to `/api/bugcheck/`, with a body containing a serialized array of bugcheck code numbers.
/// Any unknown bugcheck codes will be substituted with `null`
#[tracing::instrument(name = "bulk_bugcheck_handler", skip(state))]
pub async fn post_bugcheck_handler(
    State(state): State<AppState>,
    Json(query): Json<Vec<u64>>,
) -> Result<Json<Vec<Option<BugCheckResponse>>>, StatusCode> {
    let mut response: Vec<Option<BugCheckResponse>> = Vec::with_capacity(16);
    for entry in query {
        if let Some((name, url)) = state.bugcheck_cache.get(entry) {
            response.push({
                Some(BugCheckResponse {
                    code: entry,
                    name: name.to_string(),
                    url: url.to_string(),
                })
            });
        } else {
            response.push(None);
        }
    }
    Ok(Json(response))
}
