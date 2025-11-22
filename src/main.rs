use clap::{Parser, Subcommand};
use log::error;
use std::path::PathBuf;
mod cache;
mod cmd_raw_cache;
mod config;
mod error_codes;
mod fs_cache;
mod s3_cache;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
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
    /// Get the value associated with the given key.
    /// The key must be a lowercase hex SHA-256 hash string.
    /// This command exits with code 0 if the value was found, exit code 1 if the value was not found,
    /// and error code 2 if an error occurred.
    Get {
        /// The key to look up
        key: String,
    },
    /// Associates the given key with data from stdin
    /// The key must be a lowercase hex SHA-256 hash string.
    /// This command exits with code 0 if the value was added to all caches successfully and
    /// exit code 1 if the value was not pushed to any of the caches,
    Put {
        /// The key to associate with the data
        key: String,
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

    let config = match config::Config::from_cli_args(cli.config) {
        Ok(config) => config,
        Err(e) => {
            error!("{e}");
            std::process::exit(error_codes::CONFIG_ERROR);
        }
    };

    match &cli.command {
        Commands::Get { key } => {
            let exit_code = cmd_raw_cache::handle_get(key, &config).await;
            std::process::exit(exit_code);
        }
        Commands::Put { key } => {
            let exit_code = cmd_raw_cache::handle_put(key, &config).await;
            std::process::exit(exit_code);
        }
    }
}
