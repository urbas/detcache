use crate::config;
use crate::fs_cache;
use crate::s3_cache;
use tokio::task::JoinSet;

/// Get a value by its SHA256 hash
/// Fetches from all caches in parallel
pub async fn get(sha256_hash: &str, config: &config::Config) -> Result<Option<Vec<u8>>, String> {
    if config.caches.is_empty() {
        return Ok(None);
    }

    // Create a set of tasks for all caches
    let mut tasks = JoinSet::new();

    for (name, cache_config) in &config.caches {
        let name = name.clone();
        let cache_config = cache_config.clone();
        let sha256_hash = sha256_hash.to_string();

        tasks.spawn(async move {
            match &cache_config {
                config::CacheConfig::FS { cache_dir } => {
                    log::debug!("Trying FS cache: {name}");
                    let cache_dir = cache_dir
                        .clone()
                        .unwrap_or_else(|| config::default_cache_dir().unwrap());
                    match fs_cache::get_by_sha256_hash(&sha256_hash, &cache_dir).await {
                        Ok(Some(value)) => Some(value),
                        Ok(None) => None,
                        Err(e) => {
                            log::warn!("Error from FS cache {name}: {e}");
                            None
                        }
                    }
                }
                config::CacheConfig::S3 { .. } => {
                    log::debug!("Trying S3 cache: {name}");
                    match s3_cache::get(&sha256_hash, &cache_config).await {
                        Ok(Some(value)) => Some(value),
                        Ok(None) => None,
                        Err(e) => {
                            log::warn!("Error from S3 cache {name}: {e}");
                            None
                        }
                    }
                }
            }
        });
    }

    // Process results as they complete
    while let Some(result) = tasks.join_next().await {
        match result {
            Ok(Some(value)) => return Ok(Some(value)),
            _ => continue,
        }
    }

    Ok(None)
}

/// Store a value with its SHA256 hash
/// Stores in all caches in parallel
pub async fn put(sha256_hash: &str, value: &[u8], config: &config::Config) -> Result<(), String> {
    if config.caches.is_empty() {
        return Ok(());
    }

    let mut tasks = JoinSet::new();

    for (name, cache_config) in &config.caches {
        let name = name.clone();
        let cache_config = cache_config.clone();
        let sha256_hash = sha256_hash.to_string();
        let value = value.to_vec();

        tasks.spawn(async move {
            match &cache_config {
                config::CacheConfig::FS { cache_dir } => {
                    log::debug!("Storing in FS cache: {name}");
                    let cache_dir = cache_dir
                        .clone()
                        .unwrap_or_else(|| config::default_cache_dir().unwrap());
                    match fs_cache::put_by_sha256_hash(&sha256_hash, &value, &cache_dir).await {
                        Ok(()) => Ok(()),
                        Err(e) => {
                            log::warn!("Failed to put {sha256_hash} into FS cache {name}: {e}");
                            Err(format!("FS cache {name}: {e}"))
                        }
                    }
                }
                config::CacheConfig::S3 { .. } => {
                    log::debug!("Storing in S3 cache: {name}");
                    match s3_cache::put(&sha256_hash, &value, &cache_config).await {
                        Ok(()) => Ok(()),
                        Err(e) => {
                            log::warn!("Failed to put {sha256_hash} into S3 cache {name}: {e}");
                            Err(format!("S3 cache {name}: {e}"))
                        }
                    }
                }
            }
        });
    }

    let mut errors = Vec::new();

    while let Some(result) = tasks.join_next().await {
        match result {
            Ok(Ok(())) => {}
            Ok(Err(e)) => errors.push(e),
            Err(e) => errors.push(format!("Task panicked: {e}")),
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}
