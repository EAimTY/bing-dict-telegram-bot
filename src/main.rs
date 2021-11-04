#![feature(try_blocks)]

use crate::config::Config;
use std::env;

mod bot;
mod config;
mod handler;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    let config = match Config::parse(args) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    };

    match bot::run(&config).await {
        Ok(()) => (),
        Err(err) => eprintln!("{}", err),
    }
}
