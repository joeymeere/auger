use std::collections::HashSet;
use std::sync::Arc;

use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};

pub async fn api_key_auth(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let api_keys = ApiKeys::from_env();
    
    if req.uri().path() == "/status" {
        return Ok(next.run(req).await);
    }
    
    let api_key = req
        .headers()
        .get(header::HeaderName::from_static("x-api-key"))
        .and_then(|value| value.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    if api_keys.is_valid(api_key) {
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

#[derive(Clone, Debug)]
pub struct ApiKeys {
    keys: Arc<HashSet<String>>,
}

impl ApiKeys {
    pub fn from_env() -> Self {
        let api_keys_str = std::env::var("API_KEYS").unwrap_or_default();
        let keys: HashSet<String> = api_keys_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        let keys = if keys.is_empty() {
            let default_key = "dev-api-key".to_string();
            tracing::warn!("No API keys found in environment. Using default key for development: {}", default_key);
            let mut default_keys = HashSet::new();
            default_keys.insert(default_key);
            default_keys
        } else {
            tracing::info!("Loaded {} API keys from environment", keys.len());
            keys
        };
        
        Self {
            keys: Arc::new(keys),
        }
    }
    
    pub fn is_valid(&self, key: &str) -> bool {
        self.keys.contains(key)
    }
} 