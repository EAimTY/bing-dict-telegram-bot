use crate::config::ConfigBuilder;
pub use crate::{config::Config, handler::Handler};
use std::{env, process};

mod bot;
mod config;
mod handler;

#[tokio::main]
async fn main() {
    let args = env::args().collect::<Vec<_>>();

    let mut cfg_builder = ConfigBuilder::new();

    let cfg = match cfg_builder.parse(&args) {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("{err}");
            process::exit(1);
        }
    };

    if let Err(err) = bot::run(cfg).await {
        eprintln!("{err}");
        process::exit(1);
    }
}
