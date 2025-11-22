use serde::Deserialize;
use std::{collections::HashMap, env, fs, path::PathBuf};

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum CacheConfig {
    #[serde(rename = "fs")]
    FS { cache_dir: Option<PathBuf> },
    #[serde(rename = "s3")]
    S3 {
        bucket: String,
        region: String,
        #[serde(default)]
        profile: Option<String>,
        #[serde(default)]
        prefix_key: Option<String>,
    },
}

#[derive(Clone, Deserialize)]
pub struct Config {
    pub caches: HashMap<String, CacheConfig>,
}

pub fn default_cache_dir() -> Result<PathBuf, String> {
    match env::var("XDG_CACHE_HOME") {
        Ok(dir) => Ok(PathBuf::from(dir)),
        Err(_) => {
            let home =
                env::var("HOME").map_err(|_| "HOME environment variable not set".to_string())?;
            Ok(PathBuf::from(home).join(".cache").join("detcache"))
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Config {
            caches: HashMap::new(),
        }
    }

    pub fn from_config_file(config_path: PathBuf) -> Result<Self, String> {
        let config_content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        toml::from_str(&config_content).map_err(|e| format!("Failed to parse config file: {}", e))
    }

    pub fn from_cli_args(config_path_arg: Option<PathBuf>) -> Result<Self, String> {
        let config_path = if let Some(path) = config_path_arg {
            Some(path)
        } else {
            get_xdg_config_home()
                .map(|conf_dir| conf_dir.join("detcache").join("config.toml"))
                .filter(|conf_file| conf_file.exists())
        };

        config_path
            .map(Config::from_config_file)
            .unwrap_or(Ok(Config::new()))
    }
}

fn get_xdg_config_home() -> Option<PathBuf> {
    match env::var("XDG_CONFIG_HOME") {
        Ok(dir) => Some(PathBuf::from(dir)),
        Err(_) => match env::var("HOME") {
            Ok(home) => Some(PathBuf::from(home).join(".config")),
            Err(_) => None,
        },
    }
}
