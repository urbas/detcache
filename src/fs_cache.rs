use std::fs;
use std::io::{ErrorKind, Write};
use std::path::PathBuf;
use uuid::Uuid;

use crate::config;

pub async fn get_by_sha256_hash(
    sha256_hash: &str,
    config: &config::Config,
) -> Result<Option<Vec<u8>>, String> {
    let cached_value_file = local_fs_cache_path(sha256_hash, config)?;

    match fs::read(&cached_value_file) {
        Ok(value) => Ok(Some(value)),
        Err(err) => {
            if err.kind() == ErrorKind::NotFound {
                Ok(None)
            } else {
                Err(format!(
                    "Could not find a value associated with {sha256_hash:#?}. Error: {err:?}"
                ))
            }
        }
    }
}

pub async fn put_by_sha256_hash(
    sha256_hash: &str,
    value: &[u8],
    config: &config::Config,
) -> Result<(), String> {
    let cached_value_file = local_fs_cache_path(sha256_hash, config)?;

    if let Some(parent) = cached_value_file.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory {parent:#?}: {e}"))?;
    }

    // We want to write the cache entry atomically. That's why we write to the temporary file first and then rename it.
    let uuid = Uuid::new_v4();
    let temp_file_path = cached_value_file.with_extension(format!("tmp-{}", uuid));
    let mut file = fs::File::create(&temp_file_path)
        .map_err(|e| format!("Failed to create temporary file {temp_file_path:#?}: {e}"))?;
    file.write_all(value)
        .map_err(|e| format!("Failed to write to temporary file {temp_file_path:#?}: {e}"))?;
    file.sync_all()
        .map_err(|e| format!("Failed to sync temporary file {temp_file_path:#?}: {e}"))?;
    fs::rename(&temp_file_path, &cached_value_file)
        .map_err(|e| format!("Failed to rename temporary file to {cached_value_file:#?}: {e}"))?;

    Ok(())
}

fn local_fs_cache_path(key: &str, config: &config::Config) -> Result<PathBuf, String> {
    let first_byte = &key[0..2];
    let second_byte = &key[2..4];
    let remaining_bytes = &key[4..];

    Ok(config
        .cache_dir
        .join("nr")
        .join("kv-cache")
        .join(first_byte)
        .join(second_byte)
        .join(remaining_bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_put_and_get_by_data() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");

        let config = config::Config::with_cache_dir(temp_dir.path().to_path_buf());

        let hash = "b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c";
        let value = b"/nix/store/abcd-foo";

        let retrieved_path = get_by_sha256_hash(hash, &config)
            .await
            .expect("Failed to get value");

        assert_eq!(retrieved_path, None);

        put_by_sha256_hash(hash, value, &config)
            .await
            .expect("Failed to put value");

        let retrieved_path = get_by_sha256_hash(hash, &config)
            .await
            .expect("Failed to get value");

        assert_eq!(retrieved_path, Some(value.to_vec()));
    }
}
