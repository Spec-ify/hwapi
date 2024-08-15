//! This crate contains the Axum handlers used by the server.
use databases::cpu::Cpu;
use databases::{cpu::CpuCache, usb::UsbCache, pcie::PcieCache};
use axum::extract::Query;
use axum::http::StatusCode;
use axum::{extract::State, Json};
use log::{error, warn};
use serde::{Deserialize, Serialize};


#[derive(Clone)]
pub struct AppState {
    pub cpu_cache: CpuCache,
    pub usb_cache: UsbCache,
    pub pcie_cache: PcieCache,
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
