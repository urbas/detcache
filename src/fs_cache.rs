use std::io::Write;
use std::path::PathBuf;
use std::{env, fs};
use uuid::Uuid;

pub struct Config {
    cache_dir: PathBuf,
}

impl Config {
    pub fn new() -> Result<Self, String> {
        let cache_dir = match env::var("XDG_CACHE_HOME") {
            Ok(dir) => PathBuf::from(dir),
            Err(_) => {
                let home = env::var("HOME")
                    .map_err(|_| "HOME environment variable not set".to_string())?;
                PathBuf::from(home).join(".cache")
            }
        };

        Ok(Config { cache_dir })
    }

    pub fn with_cache_dir(cache_dir: PathBuf) -> Self {
        Config { cache_dir }
    }
}

pub fn get_by_sha256_hash(sha256_hash: &str, config: &Config) -> Result<String, String> {
    let cached_value_file = local_fs_cache_path(sha256_hash, config)?;

    fs::read_to_string(&cached_value_file)
        .map_err(|_e| format!("Could not find a value associated with {sha256_hash:#?}."))
}

pub fn put_by_sha256_hash(sha256_hash: &str, value: &str, config: &Config) -> Result<(), String> {
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
    file.write_all(value.as_bytes())
        .map_err(|e| format!("Failed to write to temporary file {temp_file_path:#?}: {e}"))?;
    file.sync_all()
        .map_err(|e| format!("Failed to sync temporary file {temp_file_path:#?}: {e}"))?;

    // Atomically rename the temporary file to the final destination
    fs::rename(&temp_file_path, &cached_value_file)
        .map_err(|e| format!("Failed to rename temporary file to {cached_value_file:#?}: {e}"))?;

    Ok(())
}

fn local_fs_cache_path(key: &str, config: &Config) -> Result<PathBuf, String> {
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
    use crate::hashing;
    use tempfile::TempDir;

    #[test]
    fn test_put_and_get_by_data() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");

        let config = Config {
            cache_dir: temp_dir.path().to_path_buf(),
        };

        let test_data = "foo";
        let value = "/nix/store/abcd-foo";

        let hash = hashing::calculate_sha256_streaming(&mut std::io::Cursor::new(test_data))
            .expect("Failed to calculate the hash.s");
        put_by_sha256_hash(&hash, value, &config).expect("Failed to put value");

        let retrieved_path = get_by_sha256_hash(&hash, &config).expect("Failed to get value");

        assert_eq!(retrieved_path, value);
    }
}
