use std::sync::Arc;

use anyhow::Result;
use chrono::Utc;
use reqwest::{Client, StatusCode};
use serde_json::Value;
use solana_sdk::pubkey::Pubkey;
use tracing::info;

#[derive(Clone)]
pub struct MinioConfig {
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,
    pub bucket_name: String,
    pub use_ssl: bool,
    pub region: String,
}

impl MinioConfig {
    pub fn from_env() -> Option<Self> {
        let endpoint = std::env::var("MINIO_ENDPOINT").ok()?;
        let access_key = std::env::var("MINIO_ACCESS_KEY").ok()?;
        let secret_key = std::env::var("MINIO_SECRET_KEY").ok()?;
        let bucket_name = std::env::var("MINIO_BUCKET_NAME").unwrap_or_else(|_| "auger-results".to_string());
        let use_ssl = std::env::var("MINIO_USE_SSL")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(true);
        let region = std::env::var("MINIO_REGION").unwrap_or_else(|_| "us-east-1".to_string());

        Some(Self {
            endpoint,
            access_key,
            secret_key,
            bucket_name,
            use_ssl,
            region,
        })
    }
}

#[derive(Clone)]
pub struct MinioStorage {
    client: Arc<Client>,
    config: MinioConfig,
}

impl MinioStorage {
    pub async fn new(config: MinioConfig) -> Result<Self> {
        let client = Client::builder()
            .build()?;

        let storage = Self {
            client: Arc::new(client),
            config,
        };

        storage.ensure_bucket_exists().await?;

        Ok(storage)
    }

    fn get_base_url(&self) -> String {
        let scheme = if self.config.use_ssl { "https" } else { "http" };
        format!("{}://{}", scheme, self.config.endpoint)
    }

    async fn ensure_bucket_exists(&self) -> Result<()> {
        let url = format!("{}/{}", self.get_base_url(), self.config.bucket_name);
        
        let response = self.client
            .head(&url)
            .send()
            .await?;
        
        if response.status() == StatusCode::OK {
            info!("Bucket {} already exists", self.config.bucket_name);
            return Ok(());
        }
        
        if response.status() != StatusCode::NOT_FOUND {
            anyhow::bail!("Failed to check if bucket exists: {}", response.status());
        }
        
        info!("Creating bucket: {}", self.config.bucket_name);
        let response = self.client
            .put(&url)
            .send()
            .await?;
        
        if !response.status().is_success() {
            anyhow::bail!("Failed to create bucket: {}", response.status());
        }
        
        Ok(())
    }

    pub async fn store_program_data(
        &self, 
        program_id: &Pubkey, 
        program_data: &[u8], 
        extraction_result: &Value
    ) -> Result<String> {
        let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
        let object_key = format!("{}/{}", program_id.to_string(), timestamp);
        
        let raw_data_key = format!("{}/raw_data.bin", object_key);
        let url = format!("{}/{}/{}", self.get_base_url(), self.config.bucket_name, raw_data_key);
        
        let response = self.client
            .put(&url)
            .body(program_data.to_vec())
            .header("Content-Type", "application/octet-stream")
            .send()
            .await?;
        
        if !response.status().is_success() {
            anyhow::bail!("Failed to upload raw data: {}", response.status());
        }
        
        info!("Stored raw program data at {}/{}", self.config.bucket_name, raw_data_key);
        
        // Store the JSON extraction result
        let json_key = format!("{}/extraction_result.json", object_key);
        let url = format!("{}/{}/{}", self.get_base_url(), self.config.bucket_name, json_key);
        let json_content = serde_json::to_string_pretty(extraction_result)?;
        
        let response = self.client
            .put(&url)
            .body(json_content)
            .header("Content-Type", "application/json")
            .send()
            .await?;
        
        if !response.status().is_success() {
            anyhow::bail!("Failed to upload JSON data: {}", response.status());
        }
        
        info!("Stored extraction result at {}/{}", self.config.bucket_name, json_key);
        
        Ok(object_key)
    }

    /// Retrieve stored extraction result from MinIO
    pub async fn get_extraction_result(&self, storage_path: &str) -> Result<Option<Value>> {
        let json_key = format!("{}/extraction_result.json", storage_path);
        let url = format!("{}/{}/{}", self.get_base_url(), self.config.bucket_name, json_key);
        
        let response = self.client
            .get(&url)
            .send()
            .await?;
        
        if response.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        
        if !response.status().is_success() {
            anyhow::bail!("Failed to download JSON data: {}", response.status());
        }
        
        let json = response.json::<Value>().await?;
        Ok(Some(json))
    }
    
    /// Retrieve stored raw program data from MinIO
    pub async fn get_program_data(&self, storage_path: &str) -> Result<Option<Vec<u8>>> {
        let raw_data_key = format!("{}/raw_data.bin", storage_path);
        let url = format!("{}/{}/{}", self.get_base_url(), self.config.bucket_name, raw_data_key);
        
        let response = self.client
            .get(&url)
            .send()
            .await?;
        
        if response.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        
        if !response.status().is_success() {
            anyhow::bail!("Failed to download raw data: {}", response.status());
        }
        
        let bytes = response.bytes().await?;
        Ok(Some(bytes.to_vec()))
    }
} 