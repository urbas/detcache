use clap::{Parser, Subcommand};
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

    let config = if let Some(cache_dir) = cli.cache_dir {
        fs_cache::Config::with_cache_dir(cache_dir)
    } else {
        match fs_cache::Config::new() {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Error initializing configuration: {e}");
                std::process::exit(1);
            }
        }
    };

    match &cli.command {
        Commands::Get { json } => {
            let hash = hashing::calculate_sha256_streaming(&mut io::stdin()).unwrap();
            match fs_cache::get_by_sha256_hash(&hash, &config) {
                Ok(value) => {
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
                    eprintln!("Error: {e}");
                    if *json {
                        let output = json!({
                            "key": hash
                        });
                        println!("{}", output);
                    }
                    std::process::exit(1);
                }
            }
        }
        Commands::Put { value, json } => {
            let hash = hashing::calculate_sha256_streaming(&mut io::stdin()).unwrap();
            match fs_cache::put_by_sha256_hash(&hash, value, &config) {
                Ok(_) => {
                    if *json {
                        let output = json!({
                            "key": hash,
                            "value": value
                        });
                        println!("{output}");
                    }
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    if *json {
                        let output = json!({
                            "key": hash
                        });
                        println!("{}", output);
                    }
                    std::process::exit(1);
                }
            }
        }
    }
}
