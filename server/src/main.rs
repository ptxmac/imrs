use std::io::{BufWriter, Cursor};
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;
use axum::body::{Body, boxed};
use axum::extract::Query;
use axum::http::{Response, StatusCode};
use axum::response::{AppendHeaders, IntoResponse};
use axum::Router;
use axum::routing::get;
use clap::Parser;
use image::{ImageBuffer, ImageFormat};
use tower::{ServiceBuilder, ServiceExt};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tokio::fs;
use imrs::{plot, tvshow};
use plotters::prelude::*;
use serde::Deserialize;
use tower_http::follow_redirect::policy::PolicyExt;


#[derive(Parser, Debug)]
#[clap(name = "server", about = "Backend server")]
struct Opt {
    #[clap(short = 'l', long = "log", default_value = "info")]
    log_level: String,

    #[clap(short = 'a', long = "addr", default_value = "::1")]
    addr: String,

    #[clap(short = 'p', long = "port", default_value = "8080")]
    port: u16,

    #[clap(long = "static-dir", default_value = "./dist")]
    static_dir: String,
}

#[derive(Deserialize)]
struct Hello {
    input: Option<String>,
}

async fn hello(query: Query<Hello>) -> impl IntoResponse {
    let Query(query) = query;
    let who = query.input.unwrap_or("Test".to_string());

    format!("Hello, {}!", who)
}

#[derive(Deserialize)]
struct TvShow {
    name: String,
}

async fn plot_tvshow(query: Query<TvShow>) -> impl IntoResponse {
    let Query(query) = query;
    let name = query.name;
    // create plot
    let results = tvshow::fetch_ratings(&name).await.unwrap();
    // in memory plot
    let mut buffer = vec![0; 1200 * 400 * 3];
    let root = BitMapBackend::with_buffer(&mut buffer, (1200, 400)).into_drawing_area();
    plot::create_plot_with_backend(root, &results.name, results.ratings).unwrap();
    // create image
    let image_buffer: ImageBuffer<image::Rgb<u8>, Vec<u8>> = ImageBuffer::from_vec(1200, 400, buffer).unwrap();

    // convert to png
    let mut buffer = BufWriter::new(Cursor::new(Vec::new()));
    image_buffer.write_to(&mut buffer, ImageFormat::Png).unwrap();
    let bytes = buffer.into_inner().unwrap().into_inner();
    (
        AppendHeaders([("Content-Type", "image/png")]),
        bytes
    )
}


#[tokio::main]
async fn main() {
    let opt = Opt::parse();

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", format!("{},hyper=info,mio=info", opt.log_level));
    }

    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/api/hello", get(hello))
        .route("/api/image", get(plot_tvshow))
        .fallback_service(get(|req| async move {
            match ServeDir::new(&opt.static_dir).oneshot(req).await {
                Ok(res) => {
                    let status = res.status();
                    match status {
                        StatusCode::NOT_FOUND => {
                            let index_path = PathBuf::from(&opt.static_dir).join("index.html");
                            let index_content = match fs::read_to_string(index_path).await {
                                Err(_) => {
                                    return Response::builder()
                                        .status(StatusCode::NOT_FOUND)
                                        .body(boxed(Body::from("index file not found")))
                                        .unwrap();
                                }
                                Ok(index_content) => index_content,
                            };
                            Response::builder()
                                .status(StatusCode::OK)
                                .body(boxed(Body::from(index_content)))
                                .unwrap()
                        }
                        _ => res.map(boxed)
                    }
                }
                Err(err) => Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(boxed(Body::from(format!("error: {err}"))))
                    .expect("error response"),
            }
        }))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

    let sock_addr = SocketAddr::from((
        IpAddr::from_str(opt.addr.as_str()).unwrap_or(IpAddr::V6(Ipv6Addr::LOCALHOST)),
        opt.port,
    ));

    log::info!("Listening on http://{}", sock_addr);

    axum::Server::bind(&sock_addr)
        .serve(app.into_make_service())
        .await
        .expect("failed");
}
