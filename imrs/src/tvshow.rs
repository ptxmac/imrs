use std::collections::HashMap;
use anyhow::{anyhow, Result};
use log::info;
use regex::Regex;
use tokio::task::JoinSet;

/*
TODO:
- reqwest client reuse
 */

#[derive(Debug, Clone)]
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

pub async fn fetch_id_and_title(name: &str) -> Result<(String, String)> {
    let url = format!("https://www.imdb.com/find?q={}&s=tt&ttype=tv", name);
    let response = reqwest::get(url).await?;
    let text = response.text().await?;

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

async fn fetch_seasons(tt_id: &str) -> Result<Vec<String>> {
    // Get seasons
    let url = format!("https://www.imdb.com/title/{}/episodes/_ajax", tt_id);
    let response = reqwest::get(url).await?;
    let text = response.text().await?;
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

async fn fetch_season_ratings(tt_id: &str, season: &str) -> Result<Vec<f32>> {
    info!("Fetch ratings for season {}", season);

    let mut season_ratings = Vec::new();

    // Get rating
    let url = format!("https://www.imdb.com/title/{}/episodes/_ajax?season={}", tt_id, season);
    let response = reqwest::get(url).await?;
    let text = response.text().await?;
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

pub async fn fetch_ratings(name: &str) -> Result<Ratings> {
    let (id, title) = fetch_id_and_title(name).await?;
    fetch_ratings_ident(&id, &title).await
}

pub async fn fetch_ratings_ident(id: & str, title: &str) -> Result<Ratings> {
    let seasons = fetch_seasons(&id).await?;

    info!("found {} seasons", seasons.len());

    let mut results = HashMap::new();

    let mut set = JoinSet::new();


    for season in seasons {
        let id = id.to_string();
        set.spawn(async move {
            let season_ratings = fetch_season_ratings(&id, &season).await;
            (season, season_ratings)
        });
    }
    while let Some(r) = set.join_next().await {
        let (season, season_ratings_result) = r?;
        let season_ratings = season_ratings_result?;
        results.insert(season, season_ratings);
    }

    Ok(Ratings {
        name: title.to_string(),
        ratings: results,
    })
}

pub fn test_ratings() -> Ratings {
    let mut result: HashMap<String, Vec<f32>> = HashMap::new();
    result.insert("1".to_string(), vec![9.0, 8.6, 8.7, 8.2, 8.3, 9.3, 8.8]);
    result.insert("2".to_string(), vec![8.6, 9.3, 8.3, 8.2, 8.3, 8.8, 8.6, 9.2, 9.1, 8.4, 8.9, 9.3, 9.2]);
    result.insert("3".to_string(), vec![8.5, 8.6, 8.4, 8.2, 8.5, 9.3, 9.6, 8.7, 8.4, 7.9, 8.4, 9.5, 9.7]);
    result.insert("4".to_string(), vec![9.2, 8.2, 8.0, 8.6, 8.6, 8.4, 8.8, 9.3, 8.8, 9.6, 9.7, 9.5, 9.9]);
    result.insert("5".to_string(), vec![9.2, 8.8, 8.8, 8.8, 9.7, 9.0, 9.5, 9.6, 9.4, 9.2, 9.6, 9.1, 9.8, 10.0, 9.7, 9.9]);

    Ratings {
        name: "Breaking Bad".to_string(),
        ratings: result,
    }
}