use std::collections::HashMap;
use plotters::prelude::*;

use anyhow::{Result};
use log::info;
use plotters::coord::Shift;

pub type Data = HashMap<String, Vec<f32>>;

pub fn create_plot(title: &str, data: Data) -> Result<()> {
    let root = BitMapBackend::new("test.png", (1200, 400)).into_drawing_area();

    create_plot_with_backend(root, title, data)
}

pub fn create_plot_with_backend(root: DrawingArea<BitMapBackend, Shift>, title: &str, data: Data) -> Result<()> {
    let total = data.iter().fold(0, |acc, v| acc + v.1.len());
    info!("total: {}", total);

    root.fill(&WHITE)?;
    let root = root.titled(format!("IMDb Ratings for {}", title).as_str(), ("sans-serif", 24))?;

    let mut chart = ChartBuilder::on(&root)
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0..total, -1.0f32..10.0f32)?;
    chart.configure_mesh()
        .x_desc("Episode")
        .y_desc("Rating")
        .light_line_style(&WHITE)
        .draw()?;

    let mut start: usize = 1;

    let mut seasons: Vec<_> = data.keys().collect();
    seasons.sort_by(|a, b| a.cmp(b));
    for (idx, season) in seasons.iter().enumerate() {
        let color = Palette99::pick(idx);
        let dot_color = color.filled();

        let ratings = data.get(*season).unwrap();
        let data: Vec<_> = ratings
            .iter()
            .enumerate()
            .map(|(i, x)| (start + i, *x))
            .collect();

        info!("season: {:?}", data);

        // Lines
        chart.draw_series(LineSeries::new(
            data.iter().map(|(x, y)| (*x, *y)),
            color.stroke_width(2),
        ))?
            .label(format!("Season {}", season))
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color.filled()));
        // Dots
        chart.draw_series(
            data.iter()
                .map(move |(x, y)|
                    Circle::new((*x, *y), 2, dot_color)
                ))?;


        start += ratings.len();
    }


    chart.configure_series_labels()
        .position(SeriesLabelPosition::LowerLeft)
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    root.present()?;

    Ok(())
}
