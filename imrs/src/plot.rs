use std::collections::HashMap;
use plotters::prelude::*;
use anyhow::{anyhow, Result};
use log::info;

pub type Data = HashMap<String, Vec<f32>>;

pub fn create_plot(data: Data) -> Result<()> {
    let total = data.iter().fold(0, |acc, v| acc + v.1.len());
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

    let mut seasons: Vec<_> = data.keys().collect();
    seasons.sort_by(|a, b| a.cmp(b));
    for (idx,season) in seasons.iter().enumerate() {
        let color = Palette99::pick(idx);

        let ratings = data.get(*season).unwrap();
        let data: Vec<_> = ratings
            .iter()
            .enumerate()
            .map(|(i, x)| (start + i, *x))
            .collect();

        info!("season: {:?}", data);

        chart.draw_series(LineSeries::new(
            data,
            color,
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
