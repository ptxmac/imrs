use clap::{Parser, Subcommand};
use log::info;
use anyhow::{Result};
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

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", format!("{}", cli.log_level));
    }

    env_logger::init();

    use Commands::*;
    match &cli.command {
        Test {} => test(),
        TV { name, } => tv_show(name).await
    }
}

fn test() -> Result<()> {
    let results = tvshow::test_ratings();
    plot::create_plot(&results.name, results.ratings)?;
    Ok(())
}

async fn tv_show(name: &str) -> Result<()> {
    info!("Looking up ratings for {}", name);

    let results = tvshow::fetch_ratings(name).await?;
    plot::create_plot(&results.name, results.ratings)?;

    Ok(())
}
