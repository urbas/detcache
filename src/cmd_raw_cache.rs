use log::{debug, error, info};
use serde_json::json;
use std::io;

use crate::{cache, config, hashing};

/// Handles the Get command
pub async fn handle_get(json: bool, config: &config::Config) -> i32 {
    let hash = hashing::calculate_sha256_streaming(&mut io::stdin()).unwrap();
    debug!("Calculated hash: {hash}");
    match cache::get(&hash, config).await {
        Ok(Some(value)) => {
            info!("Successfully retrieved value for hash {hash}");
            print_output(json, &hash, Some(&value));
            0
        }
        Ok(None) => {
            info!("Value not found for hash {hash}");
            print_output(json, &hash, None);
            1
        }
        Err(e) => {
            error!("{e}");
            print_output(json, &hash, None);
            2
        }
    }
}

/// Handles the Put command
pub async fn handle_put(value: &str, json: bool, config: &config::Config) -> i32 {
    let hash = hashing::calculate_sha256_streaming(&mut io::stdin()).unwrap();
    debug!("Calculated hash: {hash}");
    match cache::put(&hash, value, config).await {
        Ok(_) => {
            info!("Successfully stored value for hash {hash}");
            print_output(json, &hash, Some(value));
            0
        }
        Err(e) => {
            error!("{e}");
            print_output(json, &hash, None);
            1
        }
    }
}

/// Prints the output in either JSON or plain text format
pub fn print_output(json_format: bool, key: &str, value: Option<&str>) {
    if json_format {
        match value {
            Some(val) => {
                let output = json!({
                    "key": key,
                    "value": val
                });
                println!("{output}");
            }
            None => {
                let output = json!({
                    "key": key
                });
                println!("{output}");
            }
        }
    } else if let Some(val) = value {
        println!("{val}");
    }
}
