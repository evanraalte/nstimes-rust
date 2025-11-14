use clap::{Parser, Subcommand};
use dotenv::dotenv;
use nstimes::cache::PriceCache;
use nstimes::commands;

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// Enable price caching with specified file path
    #[arg(long, global = true)]
    cache: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Find train trips between two stations
    Trip {
        /// Start station name to search for
        from: String,
        /// Destination station name to search for
        to: String,
    },
    /// Get price information for a trip
    Price {
        /// Start station name to search for
        from: String,
        /// Destination station name to search for
        to: String,
        /// Travel class: 1 for first class, 2 for second class (default: 2)
        #[arg(long, value_parser = clap::value_parser!(u8).range(1..=2))]
        class: Option<u8>,
        /// Get price for return trip instead of single trip
        #[arg(long)]
        r#return: bool,
    },
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let args = Args::parse();

    // Initialize cache if --cache flag is provided
    let cache = if let Some(cache_path) = &args.cache {
        Some(PriceCache::new(cache_path)?)
    } else {
        None
    };

    match args.command {
        Commands::Trip { from, to } => commands::trip::execute(&from, &to)?,
        Commands::Price {
            from,
            to,
            class,
            r#return,
        } => {
            let travel_class = class.map(|c| {
                if c == 1 {
                    "FIRST_CLASS".to_string()
                } else {
                    "SECOND_CLASS".to_string()
                }
            });
            commands::price::execute(&from, &to, travel_class, r#return, cache.as_ref())?
        }
    }

    Ok(())
}
