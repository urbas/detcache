use clap::{Parser, Subcommand};
use log::{debug, error, info, warn};
use serde_json::json;
use std::io;
use std::path::PathBuf;
mod fs_cache;
mod hashing;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional custom cache directory
    #[arg(long)]
    cache_dir: Option<PathBuf>,

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
    /// Get the value that's associated with the data from stdin
    Get {
        /// Output the result as JSON
        #[arg(long)]
        json: bool,
    },
    /// Associates data from stdin with the given value
    Put {
        /// The value to which to associate the data
        value: String,
        /// Output the result as JSON
        #[arg(long)]
        json: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    // Initialize logger with appropriate level
    let log_level = match cli.verbose as i8 - cli.quiet as i8 {
        i8::MIN..=-2 => log::LevelFilter::Off,
        -1 => log::LevelFilter::Error,
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    env_logger::Builder::new().filter_level(log_level).init();

    let config = if let Some(cache_dir) = cli.cache_dir {
        fs_cache::Config::with_cache_dir(cache_dir)
    } else {
        match fs_cache::Config::new() {
            Ok(config) => config,
            Err(e) => {
                error!("Error initializing configuration: {e}");
                std::process::exit(1);
            }
        }
    };

    match &cli.command {
        Commands::Get { json } => {
            let hash = hashing::calculate_sha256_streaming(&mut io::stdin()).unwrap();
            debug!("Calculated hash: {hash}");
            match fs_cache::get_by_sha256_hash(&hash, &config) {
                Ok(value) => {
                    info!("Successfully retrieved value for hash {hash}");
                    if *json {
                        let output = json!({
                            "key": hash,
                            "value": value
                        });
                        println!("{output}");
                    } else {
                        println!("{value}");
                    }
                }
                Err(e) => {
                    warn!("{e}");
                    if *json {
                        let output = json!({
                            "key": hash
                        });
                        println!("{output}");
                    }
                    std::process::exit(1);
                }
            }
        }
        Commands::Put { value, json } => {
            let hash = hashing::calculate_sha256_streaming(&mut io::stdin()).unwrap();
            debug!("Calculated hash: {hash}");
            match fs_cache::put_by_sha256_hash(&hash, value, &config) {
                Ok(_) => {
                    info!("Successfully stored value for hash {hash}");
                    if *json {
                        let output = json!({
                            "key": hash,
                            "value": value
                        });
                        println!("{output}");
                    }
                }
                Err(e) => {
                    warn!("{e}");
                    if *json {
                        let output = json!({
                            "key": hash
                        });
                        println!("{output}");
                    }
                    std::process::exit(1);
                }
            }
        }
    }
}
