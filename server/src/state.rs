use crate::opt::Opt;
use anyhow::Result;
use chrono::{DateTime, Utc};
use imrs::tvshow;
use log::info;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct IdAndTitle {
    pub id: String,
    pub title: String,
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub date: DateTime<Utc>,
    pub ratings: tvshow::Ratings,
}

#[derive(Debug)]
pub struct AppState {
    pub entries: HashMap<String, Entry>,
    pub names: HashMap<String, IdAndTitle>,
    pub opt: Opt,
}

impl AppState {
    /// Get an entry if it exists and is not outdated
    pub fn check(&self, ident: &IdAndTitle) -> Option<&Entry> {
        if let Some(entry) = self.entries.get(&ident.id) {
            info!("Found entry: {:?}", entry);
            let now = Utc::now();
            let diff = now - entry.date;

            info!("age: {}", diff.num_seconds());
            if diff.num_hours() < 24 {
                return Some(entry);
            }
        }

        info!("check: {:?}", ident);
        None
    }

    /// Update and return an entry
    pub async fn update(&mut self, ident: &IdAndTitle) -> Result<&Entry> {
        // TODO: should probably do the update using channels so we don't block while one is updating

        let results = tvshow::fetch_ratings_ident(&ident.id, &ident.title).await?;

        self.entries.insert(
            ident.id.to_string(),
            Entry {
                date: Utc::now(),
                ratings: results,
            },
        );

        Ok(self.entries.get(&ident.id).unwrap())
    }

    /// Look up the IMDb id and title for a TV Show
    pub async fn get_id_and_title(&mut self, name: &str) -> Result<IdAndTitle> {
        if let Some(ident) = self.names.get(name) {
            return Ok(ident.clone());
        }

        let (id, title) = tvshow::fetch_id_and_title(name).await?;
        let ident = IdAndTitle { id, title };
        self.names.insert(name.to_string(), ident.clone());

        Ok(ident)
    }
}
