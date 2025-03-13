use crate::config;
use crate::fs_cache;
use crate::secondary_cache;

/// Get a value by its SHA256 hash
/// Fetches from both fs_cache and secondary cache in parallel, prioritizing fs_cache
pub async fn get(sha256_hash: &str, config: &config::Config) -> Result<Option<String>, String> {
    // Clone these so they can be moved into spawned tasks
    let sha256_hash_primary = sha256_hash.to_string();
    let sha256_hash_secondary = sha256_hash.to_string();
    let config_primary = config.clone();
    let config_secondary = config.clone();

    // Spawn tasks to fetch from both cache levels in parallel.
    let primary = tokio::spawn(async move {
        fs_cache::get_by_sha256_hash(&sha256_hash_primary, &config_primary).await
    });
    let secondary = tokio::spawn(async move {
        secondary_cache::get(&sha256_hash_secondary, &config_secondary).await
    });

    // We assume that the primary cache is always faster than the secondary cache. So, let check it first.
    let primary_result = primary.await.map_err(|join_error| {
        format!("Fatal error '{join_error}' while waiting for a response from the primary cache.")
    })??;

    match primary_result {
        Some(_) => Ok(primary_result),
        None => match secondary.await.map_err(|join_error| {
            format!(
                "Fatal error '{join_error}' while waiting for a response from the secondary cache."
            )
        })?? {
            None => Ok(None),
            Some(value) => {
                let _ = fs_cache::put_by_sha256_hash(sha256_hash, &value, config).await;
                Ok(Some(value))
            }
        },
    }
}

/// Store a value with its SHA256 hash
/// Stores in both the fs_cache and the secondary cache in parallel
pub async fn put(sha256_hash: &str, value: &str, config: &config::Config) -> Result<(), String> {
    let (primary_result, secondary_result) = tokio::join!(
        fs_cache::put_by_sha256_hash(sha256_hash, value, config),
        secondary_cache::put(sha256_hash, value, config)
    );

    match (&primary_result, &secondary_result) {
        (Err(e1), Err(e2)) => Err(format!(
            "Primary cache error: {}, Secondary cache error: {}",
            e1, e2
        )),
        _ => primary_result.and(secondary_result),
    }
}
