use std::collections::HashMap;
use std::io::{BufWriter, Cursor};
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc};
use axum::body::{Body, boxed};
use axum::extract::{Query, State};
use axum::http::{Response, StatusCode};
use axum::response::{AppendHeaders, IntoResponse};
use axum::{Json, Router};
use axum::routing::get;
use chrono::{DateTime, Utc};
use clap::Parser;
use image::{ImageBuffer, ImageFormat};
use log::{error, info};
use tower::{ServiceBuilder, ServiceExt};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tokio::fs;
use imrs::{plot, tvshow};
use plotters::prelude::*;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use anyhow::{Result};


#[derive(Parser, Debug, Clone)]
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

    /// The public facing URL prefix for the backend
    #[clap(long, env, default_value = "http://localhost:8080")]
    url_prefix: String,
}

#[derive(Deserialize)]
struct Hello {
    input: Option<String>,
}

async fn hello(Query(query): Query<Hello>) -> impl IntoResponse {
    let who = query.input.unwrap_or("Test".to_string());

    format!("Hello, {}!", who)
}

#[derive(Deserialize)]
struct TvShow {
    name: String,
}

async fn plot_tvshow(Query(query): Query<TvShow>, State(state): State<SharedState>) -> impl IntoResponse {
    let name = query.name;

    let ident = {
        let mut state = state.write().await;
        state.get_id_and_title(&name).await
    }.unwrap();

    let entry = {
        let mut state = state.write().await;
        match state.check(&ident) {
            Some(entry) => entry,
            None => {
                state.update(&ident).await.unwrap()
            }
        }.clone()
    };
    info!("Entry {:?}", entry);
    // create plot
    let results = entry.ratings;
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

#[derive(Debug, Deserialize)]
struct Slack {
    text: String,
    response_url: String,
}

#[derive(Serialize)]
struct SlackResponse {
    response_type: String,
    text: String,
}

#[derive(Debug, Serialize)]
struct SlackMessageAttachment {
    image_url: Option<String>,
}

#[derive(Debug, Serialize)]
struct SlackMessage {
    response_type: String,
    replace_original: bool,
    attachments: Vec<SlackMessageAttachment>,
}

async fn slack(Query(query): Query<Slack>, State(state): State<SharedState>) -> impl IntoResponse {
    info!("Slack request, {:?}", query);
    info!(" state: {:?}", state);
    let opt = {
        let state = state.read().await;
        state.opt.clone()
    };
    info!(" opts: {:?}", opt);
    let prefix = opt.url_prefix;

    tokio::spawn(async move {
        let ident = {
            let mut state = state.write().await;
            state.get_id_and_title(&query.text).await
        }.unwrap();

        info!("id: {:?}", ident);
        {
            let mut state = state.write().await;
            let entry = match state.check(&ident) {
                Some(entry) => entry,
                None => {
                    state.update(&ident).await.unwrap()
                }
            };
        }

        // send to slack
        let name = urlencoding::encode(&query.text);
        info!("encoded: {}", name);


        let m = SlackMessage {
            response_type: "in_channel".to_string(),
            replace_original: true,
            attachments: vec![
                SlackMessageAttachment {
                    image_url: Some(format!("{}/api/image?name={}", prefix, name)),
                }
            ],
        };

        info!("slack response: {:?}", m);
        let client = reqwest::Client::new();
        let resp = client.post(query.response_url)
            .json(&m)
            .send().await;
        if let Err(e) = resp {
            error!("Slack error: {}", e);
        }
    });

    Json(SlackResponse {
        response_type: "in_channel".to_string(),
        text: "Hello ".to_string(),
    })
}

type SharedState = Arc<RwLock<AppState>>;

#[derive(Clone, Debug)]
struct IdAndTitle {
    id: String,
    title: String,
}

#[derive(Debug, Clone)]
struct Entry {
    date: DateTime<Utc>,
    title: String,
    ratings: tvshow::Ratings,
}

#[derive(Debug)]
struct AppState {
    entries: HashMap<String, Entry>,
    names: HashMap<String, IdAndTitle>,
    opt: Opt,
}

impl AppState {
    fn check(&self, ident: &IdAndTitle) -> Option<&Entry> {
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

    async fn update(&mut self, ident: &IdAndTitle) -> Result<&Entry> {
        // TODO: should probably do the update using channels so we don't block while one is updating

        let results = tvshow::fetch_ratings_ident(&ident.id, &ident.title).await?;

        self.entries.insert(ident.id.to_string(), Entry {
            date: Utc::now(),
            title: ident.title.to_string(),
            ratings: results,
        });

        Ok(self.entries.get(&ident.id).unwrap())
    }

    async fn get_id_and_title(&mut self, name: &str) -> Result<IdAndTitle> {
        if let Some(ident) = self.names.get(name) {
            return Ok(ident.clone());
        }

        let (id, title) = tvshow::fetch_id_and_title(name).await?;
        let ident = IdAndTitle {
            id,
            title,
        };
        self.names.insert(name.to_string(), ident.clone());

        Ok(ident)
    }
}

#[tokio::main]
async fn main() {
    let opt = Opt::parse();

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", format!("{},hyper=info,mio=info", opt.log_level));
    }

    tracing_subscriber::fmt::init();

    let shared_state = Arc::new(RwLock::new(AppState {
        entries: HashMap::new(),
        names: HashMap::new(),
        opt: opt.clone(),
    }));


    let app = Router::new()
        .route("/api/hello", get(hello))
        .route("/api/image", get(plot_tvshow))
        .route("/api/slack", get(slack))
        .with_state(Arc::clone(&shared_state))
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

    info!("Listening on http://{}", sock_addr);

    axum::Server::bind(&sock_addr)
        .serve(app.into_make_service())
        .await
        .expect("failed");
}
