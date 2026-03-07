use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::config::{Credentials, Region};
use aws_sdk_s3::Client;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::env;

use crate::models::storage::{StorageConfigDocument, StorageProvider};

const ENCRYPTED_PREFIX: &str = "enc:v1:";

#[derive(Clone)]
pub struct ActiveStorage {
    pub provider: StorageProvider,
    pub client: Option<Client>,
    pub bucket: String,
    pub region: String,
    pub endpoint: Option<String>,
    pub quota_bytes: Option<u64>,
    pub access_key: Option<String>,
    pub secret_key: Option<String>,
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value.and_then(|v| {
        let trimmed = v.trim().to_string();
        if trimmed.is_empty() { None } else { Some(trimmed) }
    })
}

fn env_bucket() -> String {
    env::var("STORAGE_BUCKET").unwrap_or_else(|_| "khunphaen-assets".to_string())
}

fn env_region() -> String {
    env::var("STORAGE_REGION").unwrap_or_else(|_| "us-east-1".to_string())
}

fn env_endpoint() -> String {
    env::var("STORAGE_URL").unwrap_or_else(|_| "http://localhost:9000".to_string())
}

fn env_access_key() -> String {
    env::var("RUSTFS_ACCESS_KEY").unwrap_or_else(|_| "rustfsadmin".to_string())
}

fn env_secret_key() -> String {
    env::var("RUSTFS_SECRET_KEY").unwrap_or_else(|_| "rustfsadmin".to_string())
}

fn env_quota_bytes() -> Option<u64> {
    env::var("STORAGE_QUOTA_GB")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map(|gb| gb.saturating_mul(1024 * 1024 * 1024))
}

fn encryption_secret() -> String {
    env::var("STORAGE_CONFIG_ENCRYPTION_KEY")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| env::var("JWT_SECRET").ok())
        .unwrap_or_else(|| "dev-only-storage-config-key".to_string())
}

fn encryption_key_bytes() -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(encryption_secret().as_bytes());
    hasher.finalize().into()
}

fn decrypt_value_if_needed(value: Option<String>) -> Option<String> {
    let value = clean_optional(value)?;
    if !value.starts_with(ENCRYPTED_PREFIX) {
        return Some(value);
    }

    let encoded = &value[ENCRYPTED_PREFIX.len()..];
    let payload = BASE64.decode(encoded).ok()?;
    if payload.len() < 13 {
        return None;
    }

    let (nonce_bytes, ciphertext) = payload.split_at(12);
    let cipher = Aes256Gcm::new_from_slice(&encryption_key_bytes()).ok()?;
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher.decrypt(nonce, ciphertext).ok()?;
    String::from_utf8(plaintext).ok()
}

pub fn encrypt_value_if_present(value: Option<String>) -> Option<String> {
    let value = clean_optional(value)?;
    if value.starts_with(ENCRYPTED_PREFIX) {
        return Some(value);
    }

    let cipher = Aes256Gcm::new_from_slice(&encryption_key_bytes()).ok()?;
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, value.as_bytes()).ok()?;

    let mut payload = nonce_bytes.to_vec();
    payload.extend(ciphertext);
    Some(format!("{}{}", ENCRYPTED_PREFIX, BASE64.encode(payload)))
}

pub fn decrypt_storage_config(config: &StorageConfigDocument) -> StorageConfigDocument {
    let mut config = config.clone();
    config.access_key = decrypt_value_if_needed(config.access_key);
    config.secret_key = decrypt_value_if_needed(config.secret_key);
    config
}

pub fn default_storage_config_doc() -> StorageConfigDocument {
    StorageConfigDocument {
        key: "storage_config".to_string(),
        provider: StorageProvider::Env,
        bucket: Some(env_bucket()),
        region: Some(env_region()),
        endpoint: Some(env_endpoint()),
        access_key: None,
        secret_key: None,
        updated_at: None,
    }
}

pub fn resolve_active_storage(config: Option<&StorageConfigDocument>) -> ActiveStorage {
    let decrypted_config = config.map(decrypt_storage_config);
    let config = decrypted_config.as_ref();
    let provider = config
        .map(|cfg| cfg.provider.clone())
        .unwrap_or(StorageProvider::Env);

    match provider {
        StorageProvider::Env => ActiveStorage {
            provider,
            client: None,
            bucket: clean_optional(config.and_then(|cfg| cfg.bucket.clone())).unwrap_or_else(env_bucket),
            region: clean_optional(config.and_then(|cfg| cfg.region.clone())).unwrap_or_else(env_region),
            endpoint: Some(
                clean_optional(config.and_then(|cfg| cfg.endpoint.clone()))
                    .unwrap_or_else(env_endpoint),
            ),
            quota_bytes: env_quota_bytes(),
            access_key: clean_optional(config.and_then(|cfg| cfg.access_key.clone()))
                .or_else(|| Some(env_access_key())),
            secret_key: clean_optional(config.and_then(|cfg| cfg.secret_key.clone()))
                .or_else(|| Some(env_secret_key())),
        },
        StorageProvider::S3 => ActiveStorage {
            provider,
            client: None,
            bucket: clean_optional(config.and_then(|cfg| cfg.bucket.clone())).unwrap_or_else(env_bucket),
            region: clean_optional(config.and_then(|cfg| cfg.region.clone())).unwrap_or_else(env_region),
            endpoint: clean_optional(config.and_then(|cfg| cfg.endpoint.clone())),
            quota_bytes: env_quota_bytes(),
            access_key: clean_optional(config.and_then(|cfg| cfg.access_key.clone())),
            secret_key: clean_optional(config.and_then(|cfg| cfg.secret_key.clone())),
        },
    }
}

pub async fn create_s3_client(
    access_key: &str,
    secret_key: &str,
    endpoint_url: Option<&str>,
    region_name: &str,
) -> Client {
    
    let credentials = Credentials::new(access_key, secret_key, None, None, "static");

    let region_provider =
        RegionProviderChain::default_provider().or_else(Region::new(region_name.to_string()));

    let mut config_loader = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(region_provider)
        .credentials_provider(credentials);

    if let Some(endpoint_url) = endpoint_url.filter(|value| !value.trim().is_empty()) {
        config_loader = config_loader.endpoint_url(endpoint_url);
        tracing::info!(
            "🔗 Initialized S3 storage client to custom endpoint: {}",
            endpoint_url
        );
    } else {
        tracing::info!(
            "🔗 Initialized S3 storage client using AWS-managed endpoint for region: {}",
            region_name
        );
    }

    let config = config_loader.load().await;

    let s3_config = aws_sdk_s3::config::Builder::from(&config)
        .force_path_style(true)
        .build();

    Client::from_conf(s3_config)
}

pub async fn build_active_storage(config: Option<&StorageConfigDocument>) -> ActiveStorage {
    let mut active = resolve_active_storage(config);

    let credentials = active
        .access_key
        .clone()
        .zip(active.secret_key.clone());

    if let Some((access_key, secret_key)) = credentials {
        let client = create_s3_client(
            &access_key,
            &secret_key,
            active.endpoint.as_deref(),
            &active.region,
        )
        .await;
        if let Err(e) = ensure_bucket_exists(&client, &active.bucket).await {
            tracing::warn!("Failed to verify storage bucket: {}", e);
        }
        active.client = Some(client);
    } else {
        tracing::warn!("⚠️ Storage credentials are missing. Attachment capabilities will be disabled.");
    }

    active
}

pub async fn ensure_bucket_exists(
    client: &Client,
    bucket_name: &str,
) -> Result<(), aws_sdk_s3::Error> {
    let resp = client.list_buckets().send().await?;
    let buckets = resp.buckets();

    let exists = buckets.iter().any(|b| b.name() == Some(bucket_name));

    if !exists {
        tracing::info!("🪣 Bucket '{}' does not exist, creating it...", bucket_name);
        client.create_bucket().bucket(bucket_name).send().await?;
        tracing::info!("✅ Bucket '{}' created successfully", bucket_name);
    } else {
        tracing::debug!("✅ Bucket '{}' exists", bucket_name);
    }

    Ok(())
}
