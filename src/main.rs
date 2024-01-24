mod cpu;
use axum::{extract::State, routing::get, Json, Router};
use cpu::{Cpu, CpuCache};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
struct AppState {
    pub cpu_cache: CpuCache,
}

#[derive(Debug, Deserialize, Serialize)]
struct CpuQuery {
    pub name: String,
}

/// This handler accepts a json in the form of a [CpuQuery]
/// ```json
/// {
///     "name": "SEARCH_QUERY"
/// }
/// ```
/// It relies on a globally shared [AppState] to re-use the cpu cache, and responds to the request with a serialized [Cpu].
/// It will always attempt to find a cpu, and should always return a cpu. The correctness of the return value is not guaranteed.
async fn get_cpu_handler(State(state): State<AppState>, Json(query): Json<CpuQuery>) -> Json<Cpu> {
    // just to get type annotations working
    let state: AppState = state;

    Json(state.cpu_cache.find(&query.name))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/api/cpus", get(get_cpu_handler))
        .with_state(AppState {
            cpu_cache: CpuCache::new(),
        });

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
