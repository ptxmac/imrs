use std::collections::HashMap;
use clap::{Parser, Subcommand};
use log::info;
use anyhow::{anyhow, Result};
use regex::{escape, Regex};
use plotters::prelude::*;
use imrs::{plot, tvshow};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, default_value = "info")]
    log_level: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// A test command
    Test {},

    /// Look up ratings for a TV show
    TV {
        name: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", format!("{}", cli.log_level));
    }

    env_logger::init();

    use Commands::*;
    match &cli.command {
        Test {} => test(),
        TV { name, } => tv_show(name)
    }
}

fn test() -> Result<()> {
    // test plot
    let mut result: HashMap<String, Vec<f32>> = HashMap::new();
    result.insert("1".to_string(), vec![9.0, 8.6, 8.7, 8.2, 8.3, 9.3, 8.8]);
    result.insert("2".to_string(), vec![8.6, 9.3, 8.3, 8.2, 8.3, 8.8, 8.6, 9.2, 9.1, 8.4, 8.9, 9.3, 9.2]);
    result.insert("3".to_string(), vec![8.5, 8.6, 8.4, 8.2, 8.5, 9.3, 9.6, 8.7, 8.4, 7.9, 8.4, 9.5, 9.7]);
    result.insert("4".to_string(), vec![9.2, 8.2, 8.0, 8.6, 8.6, 8.4, 8.8, 9.3, 8.8, 9.6, 9.7, 9.5, 9.9]);
    result.insert("5".to_string(), vec![ 9.2, 8.8, 8.8, 8.8, 9.7, 9.0, 9.5, 9.6, 9.4, 9.2, 9.6, 9.1, 9.8, 10.0, 9.7, 9.9]);

    plot::create_plot("Breaking bad", result)?;
    Ok(())
}

fn tv_show(name: &str) -> Result<()> {
    info!("Looking up ratings for {}", name);

    let results = tvshow::fetch_ratings(name)?;
    plot::create_plot( &results.name, results.ratings)?;

    Ok(())
}
