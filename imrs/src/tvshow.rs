use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;
use anyhow::{anyhow, Context, Result};
use log::info;
use regex::Regex;


pub struct Ratings {
    pub name: String,
    pub ratings: HashMap<String, Vec<f32>>,
}


#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Todo")]
    Todo,

    #[error("Not found: {0}")]
    NotFound(String),
}

fn fetch_id_and_title(name: &str) -> Result<(String, String)> {
    let url = format!("https://www.imdb.com/find?q={}&s=tt&ttype=tv", name);
    let response = reqwest::blocking::get(url)?;
    let text = response.text()?;

    let document = scraper::Html::parse_document(&text);

    let candidate_selector = scraper::Selector::parse(".findResult .result_text a, .find-title-result a").unwrap();

    let mut candidates = document.select(&candidate_selector);

    let candidate = candidates.next().ok_or(Error::NotFound("candidate".to_string()))?;

    let link = candidate.value();
    let link = link.attr("href").ok_or(anyhow!("no link"))?;

    let tt_id_re = Regex::new(r"/title/(.+)/").unwrap();
    let cap = tt_id_re.captures(link).ok_or(anyhow!("no match"))?;
    let tt_id = cap.get(1).ok_or(anyhow!("no match"))?.as_str();


    let title = candidate.inner_html();

    Ok((tt_id.to_string(), title))
}

fn fetch_seasons(tt_id: &str) -> Result<Vec<String>> {
    // Get seasons
    let url = format!("https://www.imdb.com/title/{}/episodes/_ajax", tt_id);
    let response = reqwest::blocking::get(url)?;
    let text = response.text()?;
    let season_selector = scraper::Selector::parse("#bySeason option").unwrap();
    let document = scraper::Html::parse_document(&text);
    let seasons = document.select(&season_selector);

    let mut res = Vec::new();

    for season in seasons {
        let season = season.value().attr("value").ok_or(anyhow!("no value for season"))?;
        res.push(season.to_string());
    }
    return Ok(res);
}

fn fetch_season_ratings(tt_id: &str, season: &str) -> Result<Vec<f32>> {
    info!("Fetch ratings for season {}", season);

    let mut season_ratings = Vec::new();

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

    Ok(season_ratings)
}

pub fn fetch_ratings(name: &str) -> Result<Ratings> {
    let (id, title) = fetch_id_and_title(name)?;
    let seasons = fetch_seasons(&id)?;

    let mut results = HashMap::new();

    for season in seasons {
        let season_ratings = fetch_season_ratings(&id, &season)?;
        results.insert(season, season_ratings);
    }

    Ok(Ratings {
        name: title,
        ratings: results,
    })
}
