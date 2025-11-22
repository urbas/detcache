use log::{debug, error, info};
use std::io::{self, Read, Write};

use crate::{cache, config, error_codes};

/// Handles the Get command
pub async fn handle_get(key: &str, config: &config::Config) -> i32 {
    debug!("Looking up key: {key}");

    if let Some(exit_code) = assert_valid_sha256_hash(key) {
        return exit_code;
    }

    match cache::get(key, config).await {
        Ok(Some(value)) => {
            io::stdout().write_all(&value).unwrap_or_else(|e| {
                error!("Failed to write to stdout: {e}");
            });
            error_codes::SUCCESS
        }
        Ok(None) => {
            info!("Value not found for key {key}");
            error_codes::VALUE_NOT_FOUND
        }
        Err(e) => {
            error!("{e}");
            error_codes::CACHE_ERROR
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
        return error_codes::VALUE_NOT_FOUND;
    }

    debug!("Storing value for key: {key}");
    match cache::put(key, &buffer, config).await {
        Ok(_) => {
            info!("Successfully stored value for key {key}");
            error_codes::SUCCESS
        }
        Err(e) => {
            error!("{e}");
            error_codes::CACHE_ERROR
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
        Some(error_codes::INVALID_KEY)
    } else {
        None
    }
}
