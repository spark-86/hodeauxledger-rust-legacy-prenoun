use axum::{
    Router,
    extract::{Json, State},
    routing::post,
};
use serde::{Deserialize, Serialize};
use std::{io::Cursor, sync::Arc};

use crate::argv::ListenArgs;
use hl_core::{Rhex, from_base64};

#[derive(Deserialize)]
struct AppendRequest {
    // base64-encoded CBOR blob
    rhex_b64: String,
}

#[derive(Serialize)]
struct AppendResponse {
    ok: bool,
    rhex: Option<Vec<Rhex>>,
    current_hash: Option<String>,
    error: Option<String>,
}

async fn process(rhex: &Rhex, args: Arc<ListenArgs>) -> anyhow::Result<Vec<Rhex>> {
    // Do nothing for now
    use hl_core::{Key, keymaster::keymaster::Keymaster};
    use hl_services::{config::load_config, process as hl_process};

    let config_file = &args.config;
    let config_file = if config_file.is_some() {
        config_file.as_ref().unwrap()
    } else {
        &"config.json".to_string()
    };
    let config = Arc::new(load_config(config_file)?);

    let mut keymaster = Keymaster::new();
    for key in config.hot_keys.iter() {
        keymaster.hot_keys.push(Key::from_bytes(*key));
    }

    let processed_rhex = hl_process::process_rhex(rhex, true, &config, &keymaster)?;
    // For now, we assume only one Rhex is returned
    // In a real scenario, you might need to handle multiple Rhex outputs
    if processed_rhex.len() == 0 {
        Err(anyhow::anyhow!("No Rhex returned from processing"))
    } else {
        Ok(processed_rhex)
    }
}

async fn append_handler(
    State(listen_args): State<Arc<ListenArgs>>,
    Json(req): Json<AppendRequest>,
) -> Json<AppendResponse> {
    // Step 1: decode base64
    let raw_bytes = match from_base64(&req.rhex_b64) {
        Ok(b) => b,
        Err(e) => {
            return Json(AppendResponse {
                ok: false,
                rhex: None,
                current_hash: None,
                error: Some(format!("base64 decode error: {e}")),
            });
        }
    };

    // Step 2: decode CBOR -> Rhex using ciborium
    let rhex: Rhex = match ciborium::de::from_reader(Cursor::new(&raw_bytes)) {
        Ok(r) => r,
        Err(e) => {
            return Json(AppendResponse {
                ok: false,
                rhex: None,
                current_hash: None,
                error: Some(format!("CBOR decode error: {e}")),
            });
        }
    };

    // Step 3: process
    match process(&rhex, listen_args).await {
        Ok(r) => {
            // serialize hash back to base64

            Json(AppendResponse {
                ok: true,
                current_hash: None,
                rhex: Some(r),
                error: None,
            })
        }
        Err(e) => Json(AppendResponse {
            ok: false,
            rhex: None,
            current_hash: None,
            error: Some(e.to_string()),
        }),
    }
}

pub async fn start_http_server(listen_args: ListenArgs) {
    let shared = Arc::new(listen_args);

    let cors = tower_http::cors::CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    let app = Router::new()
        .route("/append", post(append_handler))
        .with_state(shared)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:1978").await.unwrap();
    println!(
        "🛰️ usherd REST listening on {}",
        listener.local_addr().unwrap()
    );
    axum::serve(listener, app).await.unwrap();
}
