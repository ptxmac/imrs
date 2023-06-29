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
    let mut result:HashMap<String, Vec<f32>> = HashMap::new();
    result.insert("1".to_string(), vec![1.0, 2.0, 3.0]);
    result.insert("2".to_string(), vec![4.0, 5.0, 6.0]);

    plot::create_plot(result)?;
    Ok(())
}

fn tv_show(name: &str) -> Result<()> {
    info!("Looking up ratings for {}", name);

    let results = tvshow::fetch_ratings(name)?;
    plot::create_plot(results)?;

    Ok(())
}
