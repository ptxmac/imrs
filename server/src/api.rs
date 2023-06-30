use crate::SharedState;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;

pub mod image;
pub mod slack;

pub async fn names(State(state): State<SharedState>) -> impl IntoResponse {
    let names: Vec<_> = {
        let state = state.read().await;
        state.names.clone().keys().map(|k| k.to_string()).collect()
    };
    Json(names)
}
