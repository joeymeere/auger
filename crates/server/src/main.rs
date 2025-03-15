use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    middleware,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use tokio::signal;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, warn, Level};
use tracing_subscriber::{filter, prelude::*};

use auger::{extract_from_bytes, ExtractConfig};

use auger_server::{
    api_key_auth, 
    log_request, 
    MinioConfig, 
    MinioStorage,
    utils::process_dump
};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            filter::Targets::new()
                .with_target("auger_server", Level::DEBUG)
                .with_target("tower_http", Level::INFO)
                .with_target("axum", Level::INFO),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_timer(tracing_subscriber::fmt::time::UtcTime::rfc_3339())
                .with_ansi(true)
                .with_thread_ids(true)
                .with_thread_names(true)
        )
        .init();

    info!("Starting Auger API");

    let rpc_url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());
    let rpc_client = Arc::new(RpcClient::new_with_timeout(
        rpc_url,
        Duration::from_secs(30),
    ));

    let minio_storage = match MinioConfig::from_env() {
        Some(config) => {
            info!("Initializing MinIO storage with endpoint: {}", config.endpoint);
            match MinioStorage::new(config).await {
                Ok(storage) => {
                    info!("MinIO storage initialized successfully");
                    Some(storage)
                }
                Err(err) => {
                    warn!("Failed to initialize MinIO storage: {}", err);
                    None
                }
            }
        }
        None => {
            info!("MinIO storage not configured, storage feature will be disabled");
            None
        }
    };

    let app = Router::new()
        .route("/status", get(status_handler))
        .route("/destructure", get(destructure_handler))
        .route("/storage/:path", get(storage_handler))
        .with_state(AppState { 
            rpc_client,
            minio_storage,
        })
        .layer(middleware::from_fn(api_key_auth))
        .layer(middleware::from_fn(log_request))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], 8180));
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

#[derive(Clone)]
struct AppState {
    rpc_client: Arc<RpcClient>,
    minio_storage: Option<MinioStorage>,
}

#[derive(Serialize)]
struct StatusResponse {
    status: String,
    timestamp: DateTime<Utc>,
    storage_enabled: bool,
}

async fn status_handler(state: axum::extract::State<AppState>) -> Json<StatusResponse> {
    Json(StatusResponse {
        status: "ok".to_string(),
        timestamp: Utc::now(),
        storage_enabled: state.minio_storage.is_some(),
    })
}

#[derive(Deserialize)]
struct DestructureQuery {
    program_id: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize)]
struct DestructureResponse {
    result: serde_json::Value,
    storage_path: Option<String>,
}

async fn destructure_handler(
    Query(params): Query<DestructureQuery>,
    state: axum::extract::State<AppState>,
) -> Result<Json<DestructureResponse>, AppError> {
    let program_id = params
        .program_id
        .parse::<Pubkey>()
        .map_err(|e| AppError::BadRequest(format!("Invalid program ID: {}", e)))?;

    let program_data =
        process_dump(&state.rpc_client, Some(program_id)).expect("Failed to fetch program data");

    let extract_result = extract_from_bytes(
        program_data.as_slice(),
        Some(ExtractConfig {
            ff_sequence_length: 128,
            program_header_index: 0,
            replace_non_printable: true,
        }),
    )
    .map_err(|e| AppError::InternalError(format!("Failed to extract data: {:?}", e)))?;

    let mut result = serde_json::to_value(extract_result)
        .map_err(|e| AppError::InternalError(format!("Failed to serialize result: {}", e)))?;

    result
        .as_object_mut()
        .expect("Failed to convert to object")
        .remove("text")
        .expect("Failed to remove raw text");

    // result.as_object_mut().expect("Failed to convert to object").remove("text");

    let storage_path = if let Some(storage) = &state.minio_storage {
        match storage.store_program_data(&program_id, &program_data, &result).await {
            Ok(path) => {
                info!("Stored program data and extraction result in MinIO at path: {}", path);
                Some(path)
            }
            Err(err) => {
                warn!("Failed to store program data in MinIO: {}", err);
                None
            }
        }
    } else {
        None
    };

    Ok(Json(DestructureResponse {
        result,
        storage_path,
    }))
}

#[derive(Serialize)]
struct StorageResponse {
    extraction_result: Option<serde_json::Value>,
}

async fn storage_handler(
    Path(path): Path<String>,
    state: axum::extract::State<AppState>,
) -> Result<Json<StorageResponse>, AppError> {
    if let Some(storage) = &state.minio_storage {
        match storage.get_extraction_result(&path).await {
            Ok(Some(result)) => {
                return Ok(Json(StorageResponse {
                    extraction_result: Some(result),
                }));
            }
            Ok(None) => {
                return Err(AppError::NotFound(format!("Storage path not found: {}", path)));
            }
            Err(err) => {
                return Err(AppError::InternalError(format!("Failed to retrieve storage data: {}", err)));
            }
        }
    } else {
        return Err(AppError::InternalError("Storage is not configured".to_string()));
    }
}

enum AppError {
    BadRequest(String),
    InternalError(String),
    NotFound(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
        };

        let body = Json(ErrorResponse {
            error: error_message,
        });

        (status, body).into_response()
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Shutdown signal received, starting graceful shutdown");
}
