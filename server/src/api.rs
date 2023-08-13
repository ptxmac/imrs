use crate::SharedState;
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use std::time::Duration;
use tokio::time::sleep;
use tracing::info;

pub mod image;
pub mod slack;

#[tracing::instrument]
pub async fn names(State(state): State<SharedState>) -> impl IntoResponse {
    let names: Vec<_> = {
        let state = state.read().await;
        state.names.clone().keys().map(|k| k.to_string()).collect()
    };
    info!(?names, "fetch names");
    Json(names)
}

#[derive(Debug, Deserialize)]
pub struct Hello {
    input: Option<String>,
}

struct Thing {}

impl Drop for Thing {
    fn drop(&mut self) {
        info!("dropped a thing!");
    }
}

#[tracing::instrument]
pub async fn hello(Query(query): Query<Hello>) -> impl IntoResponse {
    let _t = Thing {};
    let who = query.input.unwrap_or("Test".to_string());
    info!("start");
    sleep(Duration::from_secs(2)).await;
    info!("Done");

    format!("Hello, {}!", who)
}
