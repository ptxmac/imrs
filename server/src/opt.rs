use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[clap(name = "server", about = "Backend server")]
pub struct Opt {
    #[clap(short = 'l', long = "log", default_value = "info")]
    pub log_level: String,

    #[clap(short = 'a', long = "addr", default_value = "::1")]
    pub addr: String,

    #[clap(short = 'p', long = "port", default_value = "8080")]
    pub port: u16,

    #[clap(long = "static-dir", default_value = "./dist")]
    pub static_dir: String,

    /// The public facing URL prefix for the backend
    #[clap(long, env, default_value = "http://localhost:8080")]
    pub url_prefix: String,
}
