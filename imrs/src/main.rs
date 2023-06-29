use std::collections::HashMap;
use clap::{Parser, Subcommand};
use log::info;
use anyhow::{anyhow, Result};
use regex::{escape, Regex};
use plotters::prelude::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, default_value = "debug")]
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
    let mut result = HashMap::new();
    result.insert("1", vec![1.0, 2.0, 3.0]);
    result.insert("2", vec![4.0, 5.0, 6.0]);

    let total = result.iter().fold(0, |acc, v| acc + v.1.len());
    info!("total: {}", total);

    info!("test plot");
    let root = BitMapBackend::new("test.png", (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Test", ("sans-serif", 50))
        .margin(5)
        .build_cartesian_2d(0..total, -1.0..10.0)?;
    chart.configure_mesh().draw()?;

    let mut start: usize = 0;

    let mut seasons: Vec<_> = result.keys().collect();
    seasons.sort_by(|a, b| a.cmp(b));
    for season in seasons {
        let ratings = result.get(season).unwrap();
        let data: Vec<_> = ratings
            .iter()
            .enumerate()
            .map(|(i, x)| (start + i, *x))
            .collect();

        info!("season: {:?}", data);

        chart.draw_series(LineSeries::new(
            data,
            &RED,
        ))?
            .label(format!("Season {}", season));
        start += ratings.len();
    }


    chart.configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    root.present()?;


    Ok(())
}

fn tv_show(name: &str) -> Result<()> {
    info!("Looking up ratings for {}", name);
    let url = format!("https://www.imdb.com/find?q={}&s=tt&ttype=tv", name);
    let response = reqwest::blocking::get(url)?;
    let text = response.text()?;

    let document = scraper::Html::parse_document(&text);

    let candidate_selector = scraper::Selector::parse(".findResult .result_text a, .find-title-result a").unwrap();

    let mut candidates = document.select(&candidate_selector);

    let candidate = candidates.next().ok_or(anyhow!("no candidate"))?;

    let link = candidate.value();
    let link = link.attr("href").ok_or(anyhow!("no link"))?;

    let tt_id_re = Regex::new(r"/title/(.+)/").unwrap();
    let cap = tt_id_re.captures(link).ok_or(anyhow!("no match"))?;
    let tt_id = cap.get(1).ok_or(anyhow!("no match"))?.as_str();


    println!("{:?}", tt_id);

    let title = candidate.inner_html();

    // Get seasons
    let url = format!("https://www.imdb.com/title/{}/episodes/_ajax", tt_id);
    let response = reqwest::blocking::get(url)?;
    let text = response.text()?;
    let season_selector = scraper::Selector::parse("#bySeason option").unwrap();
    let document = scraper::Html::parse_document(&text);
    let seasons = document.select(&season_selector);

    let mut results = HashMap::new();

    for season in seasons {
        let mut season_ratings = Vec::new();

        let season = season.value().attr("value").ok_or(anyhow!("no value for season"))?;

        // Get rating
        let url = format!("https://www.imdb.com/title/{}/episodes/_ajax?season={}", tt_id, season);
        let response = reqwest::blocking::get(url)?;
        let text = response.text()?;
        let document = scraper::Html::parse_document(&text);
        let rows_selector = scraper::Selector::parse(".info").unwrap();
        let rows = document.select(&rows_selector);

        for row in rows {
            let ep_number_selector = scraper::Selector::parse("[itemprop=\"episodeNumber\"]").unwrap();
            let ep_number = row.select(&ep_number_selector).next().ok_or(anyhow!("no ep number"))?;
            let ep_number = ep_number.value().attr("content").ok_or(anyhow!("no ep number"))?;


            let rating_widget_selector = scraper::Selector::parse(".ipl-rating-widget").unwrap();
            let rating_star_placeholder_selector = scraper::Selector::parse(".ipl-rating-star--placeholder").unwrap();
            let rating_star_selector = scraper::Selector::parse(".ipl-rating-star__rating").unwrap();
            // TODO: check missing
            if row.select(&rating_widget_selector).next() == None {
                // Not aired yet
                //season_ratings.push(-1.0);
                continue;
            }
            if row.select(&rating_star_placeholder_selector).next() != None {
                info!("no rating for {}", ep_number);
                season_ratings.push(-1.0);
                continue;
            }

            let ep_rating = row.select(&rating_star_selector).next().ok_or(anyhow!("no rating"))?;
            let ep_rating = ep_rating.inner_html();
            let ep_rating: f32 = ep_rating.parse()?;

            season_ratings.push(ep_rating);
        }
        if !season_ratings.is_empty() {
            results.insert(season, season_ratings);
        }
    }

    println!("{:?}", results);

    // create graph

    let total = results.iter().fold(0, |acc, v| acc + v.1.len());
    info!("total: {}", total);

    info!("test plot");
    let root = BitMapBackend::new("test.png", (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Test", ("sans-serif", 50))
        .margin(5)
        .build_cartesian_2d(0..total, -1.0f32..10.0f32)?;
    chart.configure_mesh().draw()?;

    let mut start: usize = 0;

    let mut seasons: Vec<_> = results.keys().collect();
    seasons.sort_by(|a, b| a.cmp(b));
    for season in seasons {
        let ratings = results.get(season).unwrap();
        let data: Vec<_> = ratings
            .iter()
            .enumerate()
            .map(|(i, x)| (start + i, *x))
            .collect();

        info!("season: {:?}", data);

        chart.draw_series(LineSeries::new(
            data,
            &RED,
        ))?
            .label(format!("Season {}", season));
        start += ratings.len();
    }


    chart.configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    root.present()?;



    Ok(())
}