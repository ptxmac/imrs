use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::str::FromStr;
use axum::handler::HandlerWithoutStateExt;
use axum::response::IntoResponse;
use axum::Router;
use axum::routing::get;
use clap::Parser;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing_subscriber::fmt::layer;

#[derive(Parser, Debug)]
#[clap(name = "server", about = "Backend server")]
struct Opt {

    #[clap(short = 'l', long = "log", default_value = "debug")]
    log_level: String,

    #[clap(short = 'a', long = "addr", default_value = "::1")]
    addr: String,

    #[clap(short = 'p', long = "port", default_value = "8080")]
    port: u16,
}

#[tokio::main]
async fn main() {
    let opt = Opt::parse();

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", format!("{},hyper=info,mio=info", opt.log_level));
    }

    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(hello))
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

async fn hello() -> impl IntoResponse {
    "Hello, world?"
}