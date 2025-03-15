use std::time::Instant;
use std::sync::atomic::{AtomicU64, Ordering};

use axum::{
    body::{Body, Bytes},
    extract::Request,
    middleware::Next,
    response::Response,
};
use tracing::{debug, info, warn};

static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);

pub async fn log_request(req: Request, next: Next) -> Response {
    let request_id = format!("req-{}", REQUEST_COUNTER.fetch_add(1, Ordering::SeqCst));
    
    let method = req.method().clone();
    let uri = req.uri().clone();
    let path = uri.path().to_string();
    let query = uri.query().unwrap_or("").to_string();
    let headers = format!("{:?}", req.headers());
    
    info!(
        request_id = %request_id,
        "REQUEST: {} {} Query: {} Headers: {}",
        method, path, query, headers
    );
    
    let start = Instant::now();
    let response = next.run(req).await;
    
    let duration = start.elapsed();
    
    let status = response.status();
    info!(
        request_id = %request_id,
        "RESPONSE: {} for {} {} - Completed in {:?}",
        status, method, path, duration
    );
    
    if status.is_client_error() || status.is_server_error() {
        warn!(
            request_id = %request_id,
            "ERROR: {} for {} {} - Completed in {:?}",
            status, method, path, duration
        );
    }
    
    response
}

// only use this when needed, performance kinda blows
pub async fn log_request_with_body(req: Request, next: Next) -> Response {
    let request_id = format!("req-{}", REQUEST_COUNTER.fetch_add(1, Ordering::SeqCst));
    
    let method = req.method().clone();
    let uri = req.uri().clone();
    let path = uri.path().to_string();
    let query = uri.query().unwrap_or("").to_string();
    let headers = format!("{:?}", req.headers());
    
    let (parts, body) = req.into_parts();
    let bytes = buffer_and_print(&request_id, "REQUEST BODY", body).await;
    let req = Request::from_parts(parts, Body::from(bytes));
    
    info!(
        request_id = %request_id,
        "REQUEST: {} {} Query: {} Headers: {}",
        method, path, query, headers
    );
    
    let start = Instant::now();
    let res = next.run(req).await;
    
    let duration = start.elapsed();
    
    let (parts, body) = res.into_parts();
    let bytes = buffer_and_print(&request_id, "RESPONSE BODY", body).await;
    let res = Response::from_parts(parts, Body::from(bytes));
    
    let status = res.status();
    info!(
        request_id = %request_id,
        "RESPONSE: {} for {} {} - Completed in {:?}",
        status, method, path, duration
    );
    
    if status.is_client_error() || status.is_server_error() {
        warn!(
            request_id = %request_id,
            "ERROR: {} for {} {} - Completed in {:?}",
            status, method, path, duration
        );
    }
    
    res
}

async fn buffer_and_print(request_id: &str, direction: &str, body: Body) -> Bytes {
    let bytes = axum::body::to_bytes(body, 1024 * 1024).await.unwrap_or_default();
    
    match std::str::from_utf8(&bytes) {
        Ok(string) => {
            let truncated = if string.len() > 1000 {
                format!("{}... (truncated, total size: {} bytes)", &string[..1000], bytes.len())
            } else {
                string.to_string()
            };
            debug!(
                request_id = %request_id,
                "{}: {}", direction, truncated
            );
        }
        Err(_) => {
            debug!(
                request_id = %request_id,
                "{}: Binary data, size: {} bytes", direction, bytes.len()
            );
        }
    }
    
    bytes
} 