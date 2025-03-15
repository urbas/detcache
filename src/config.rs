use log::error;
use serde::Deserialize;
use std::{collections::HashMap, env, fs, path::PathBuf};

#[derive(Clone, Debug, Deserialize)]
pub struct SecondaryCacheConfig {
    #[serde(rename = "type")]
    pub cache_type: String,
    pub config: toml::Table,
}

#[derive(Clone)]
pub struct Config {
    pub cache_dir: PathBuf,
    pub secondary_cache: HashMap<String, SecondaryCacheConfig>,
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

        Ok(Config {
            cache_dir,
            secondary_cache: HashMap::new(),
        })
    }

    pub fn with_cache_dir(cache_dir: PathBuf) -> Self {
        Config {
            cache_dir,
            secondary_cache: HashMap::new(),
        }
    }

    pub fn with_config_file(mut self, config_path: PathBuf) -> Result<Self, String> {
        #[derive(Deserialize)]
        struct ConfigFile {
            secondary_cache: Option<HashMap<String, SecondaryCacheConfig>>,
        }

        let config_content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let config_file: ConfigFile = toml::from_str(&config_content)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;

        if let Some(secondary_cache) = config_file.secondary_cache {
            self.secondary_cache = secondary_cache;
        }

        Ok(self)
    }

    pub fn from_cli_args(cache_dir_arg: Option<PathBuf>, config_path_arg: Option<PathBuf>) -> Self {
        let mut config = if let Some(cache_dir) = cache_dir_arg {
            Config::with_cache_dir(cache_dir)
        } else {
            match Config::new() {
                Ok(config) => config,
                Err(e) => {
                    error!("{e}");
                    std::process::exit(1);
                }
            }
        };

        let config_path = if let Some(path) = config_path_arg {
            Some(path)
        } else {
            get_xdg_config_home()
                .map(|conf_dir| conf_dir.join("detcache").join("config.toml"))
                .filter(|conf_file| conf_file.exists())
        };

        if let Some(config_path) = config_path {
            match config.with_config_file(config_path) {
                Ok(loaded_config) => config = loaded_config,
                Err(e) => {
                    error!("{e}");
                    std::process::exit(1);
                }
            }
        }

        config
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
