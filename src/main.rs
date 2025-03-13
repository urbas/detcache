use clap::{Parser, Subcommand};
use log::{debug, error, info};
use serde_json::json;
use std::io;
use std::path::PathBuf;
mod cache;
mod config;
mod fs_cache;
mod hashing;
mod s3_cache;
mod secondary_cache;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional custom cache directory
    #[arg(long)]
    cache_dir: Option<PathBuf>,

    #[arg(long)]
    config: Option<PathBuf>,

    /// Increase verbosity level (can be used multiple times)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Decrease verbosity level (can be used multiple times)
    #[arg(short, long, action = clap::ArgAction::Count)]
    quiet: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get the value that's associated with the data from stdin.
    /// This command exits with code 0 if the value was found, exit code 1 if the value was found,
    /// and error code 2 if an error occurred.
    Get {
        /// Output the result as JSON
        #[arg(long)]
        json: bool,
    },
    /// Associates data from stdin with the given value
    /// This command exits with code 0 if the value was added to all caches successfully and
    /// exit code 1 if the value was not pushed to any of the caches,
    Put {
        /// The value to which to associate the data
        value: String,
        /// Output the result as JSON
        #[arg(long)]
        json: bool,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let log_level = match cli.verbose as i8 - cli.quiet as i8 {
        i8::MIN..=-2 => log::LevelFilter::Off,
        -1 => log::LevelFilter::Error,
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    env_logger::Builder::new().filter_level(log_level).init();

    let mut config = if let Some(cache_dir) = cli.cache_dir {
        config::Config::with_cache_dir(cache_dir)
    } else {
        match config::Config::new() {
            Ok(config) => config,
            Err(e) => {
                error!("{e}");
                std::process::exit(1);
            }
        }
    };

    if let Some(config_path) = cli.config {
        match config.with_config_file(config_path) {
            Ok(loaded_config) => config = loaded_config,
            Err(e) => {
                error!("{e}");
                std::process::exit(1);
            }
        }
    }

    match &cli.command {
        Commands::Get { json } => {
            let hash = hashing::calculate_sha256_streaming(&mut io::stdin()).unwrap();
            debug!("Calculated hash: {hash}");
            match cache::get(&hash, &config).await {
                Ok(Some(value)) => {
                    info!("Successfully retrieved value for hash {hash}");
                    print_output(*json, &hash, Some(&value));
                    std::process::exit(0);
                }
                Ok(None) => {
                    info!("Value not found for hash {hash}");
                    print_output(*json, &hash, None);
                    std::process::exit(1);
                }
                Err(e) => {
                    error!("{e}");
                    print_output(*json, &hash, None);
                    std::process::exit(2);
                }
            }
        }
        Commands::Put { value, json } => {
            let hash = hashing::calculate_sha256_streaming(&mut io::stdin()).unwrap();
            debug!("Calculated hash: {hash}");
            match cache::put(&hash, value, &config).await {
                Ok(_) => {
                    info!("Successfully stored value for hash {hash}");
                    print_output(*json, &hash, Some(value));
                    std::process::exit(0);
                }
                Err(e) => {
                    error!("{e}");
                    print_output(*json, &hash, None);
                    std::process::exit(1);
                }
            }
        }
    }
}

/// Prints the output in either JSON or plain text format
fn print_output(json_format: bool, key: &str, value: Option<&str>) {
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
