use crate::{config::Config, handler::Handler};
use tgbot::{
    longpoll::LongPoll,
    methods::{GetMe, SetMyCommands},
    types::BotCommand,
    webhook::{self, HyperError},
    Api, ApiError, Config as ApiConfig, ExecuteError, ParseProxyError,
};
use thiserror::Error;

pub async fn run(config: &Config) -> Result<(), Error> {
    let mut api_config = ApiConfig::new(&config.token);

    if let Some(proxy) = &config.proxy {
        api_config = api_config.proxy(proxy)?;
    }

    let api = Api::new(api_config)?;

    let bot_info = api.execute(GetMe).await?;

    let commands = vec![
        BotCommand::new("dict", "[word / phrase] - Translate a word or phrase"),
        BotCommand::new("toggle_command", "Toggle translate-all-messages mode for the current chat (default: off)"),
        BotCommand::new("toggle_mention", "Toggle if I should only react to non-command messages that mentions. This only works in groups. You still need to @ me when using command (default: on)"),
        BotCommand::new("about", "About this bot"),
        BotCommand::new("help", "Get this help message"),
    ].into_iter().flatten();

    api.execute(SetMyCommands::new(commands)).await?;

    if config.webhook == 0 {
        println!("Running in longpoll mode");
        LongPoll::new(api.clone(), Handler::new(api, bot_info))
            .run()
            .await;
    } else {
        println!("Running in webhook mode on port {}", config.webhook);
        webhook::run_server(
            ([0, 0, 0, 0], config.webhook),
            "/",
            Handler::new(api, bot_info),
        )
        .await?;
    }

    Ok(())
}

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Api(#[from] ApiError),
    #[error(transparent)]
    Execute(#[from] ExecuteError),
    #[error(transparent)]
    Hyper(#[from] HyperError),
    #[error(transparent)]
    ParseProxy(#[from] ParseProxyError),
}
