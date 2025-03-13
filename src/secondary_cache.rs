use crate::config;
use crate::s3_cache;

/// Get a value by its SHA256 hash from the secondary cache
pub async fn get(sha256_hash: &str, config: &config::Config) -> Result<Option<String>, String> {
    for (name, cache_config) in &config.secondary_cache {
        match cache_config.cache_type.as_str() {
            "s3" => {
                log::debug!("Trying S3 secondary cache: {}", name);
                match s3_cache::get(sha256_hash, cache_config).await {
                    Ok(Some(value)) => return Ok(Some(value)),
                    Ok(None) => continue,
                    Err(e) => log::warn!("Error from S3 cache {}: {}", name, e),
                }
            }
            unknown_type => {
                log::warn!("Unknown secondary cache type: {}", unknown_type);
            }
        }
    }
    Ok(None)
}

/// Store a value with its SHA256 hash in the secondary cache
pub async fn put(sha256_hash: &str, value: &str, config: &config::Config) -> Result<(), String> {
    for (name, cache_config) in &config.secondary_cache {
        match cache_config.cache_type.as_str() {
            "s3" => {
                log::debug!("Storing in S3 secondary cache: {}", name);
                if let Err(e) = s3_cache::put(sha256_hash, value, cache_config).await {
                    log::warn!("Error storing in S3 cache {}: {}", name, e);
                }
            }
            unknown_type => {
                log::warn!("Unknown secondary cache type: {}", unknown_type);
            }
        }
    }

    Ok(())
}
