use log::{debug, error, info};
use std::io::{self, Read, Write};

use crate::{cache, config};

/// Handles the Get command
pub async fn handle_get(key: &str, config: &config::Config) -> i32 {
    debug!("Looking up key: {key}");

    if let Some(exit_code) = assert_valid_sha256_hash(key) {
        return exit_code;
    }

    match cache::get(key, config).await {
        Ok(Some(value)) => {
            info!("Successfully retrieved value for key {key}");
            io::stdout().write_all(&value).unwrap_or_else(|e| {
                error!("Failed to write to stdout: {e}");
            });
            0
        }
        Ok(None) => {
            info!("Value not found for key {key}");
            1
        }
        Err(e) => {
            error!("{e}");
            2
        }
    }
}

/// Handles the Put command
pub async fn handle_put(key: &str, config: &config::Config) -> i32 {
    if let Some(exit_code) = assert_valid_sha256_hash(key) {
        return exit_code;
    }

    let mut buffer = Vec::new();
    if let Err(e) = io::stdin().read_to_end(&mut buffer) {
        error!("Failed to read from stdin: {e}");
        return 1;
    }

    debug!("Storing value for key: {key}");
    match cache::put(key, &buffer, config).await {
        Ok(_) => {
            info!("Successfully stored value for key {key}");
            0
        }
        Err(e) => {
            error!("{e}");
            1
        }
    }
}

/// Validates that the provided key is a valid lowercase hex SHA-256 hash
/// Returns Some(exit_code) if validation fails, None if validation succeeds
fn assert_valid_sha256_hash(key: &str) -> Option<i32> {
    let is_valid = key.len() == 64 && key.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f'));

    if !is_valid {
        error!(
            "Invalid key format: {key}. Key must be a lowercase hex SHA-256 hash (64 characters)"
        );
        Some(3)
    } else {
        None
    }
}
