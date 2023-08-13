use crate::SharedState;
use axum::extract::{Query, State};
use axum::response::{AppendHeaders, IntoResponse};
use image::{ImageBuffer, ImageFormat};
use imrs::plot;
use plotters::prelude::*;
use serde::Deserialize;
use std::io::{BufWriter, Cursor};
use tracing::info;

#[derive(Deserialize)]
pub struct TvShow {
    name: String,
}

pub async fn plot_tvshow(
    Query(query): Query<TvShow>,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    let name = query.name;

    let ident = {
        let mut state = state.write().await;
        state.get_id_and_title(&name).await
    }
    .unwrap();

    let entry = {
        let mut state = state.write().await;
        match state.check(&ident) {
            Some(entry) => entry,
            None => state.update(&ident).await.unwrap(),
        }
        .clone()
    };
    info!("Entry {:?}", entry);
    // create plot
    let results = entry.ratings;
    // in memory plot
    let mut buffer = vec![0; 1200 * 400 * 3];
    {
        let root = BitMapBackend::with_buffer(&mut buffer, (1200, 400)).into_drawing_area();
        plot::create_plot_with_backend(&root, &results.name, results.ratings).unwrap();
    }

    // create image
    let image_buffer: ImageBuffer<image::Rgb<u8>, Vec<u8>> =
        ImageBuffer::from_vec(1200, 400, buffer).unwrap();

    // convert to png
    let mut buffer = BufWriter::new(Cursor::new(Vec::new()));
    image_buffer
        .write_to(&mut buffer, ImageFormat::Png)
        .unwrap();
    let bytes = buffer.into_inner().unwrap().into_inner();

    (AppendHeaders([("Content-Type", "image/png")]), bytes)
}
