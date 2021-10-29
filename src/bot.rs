use crate::config::Config;
use futures_util::future::BoxFuture;
use std::{collections::HashSet, sync::Arc};
use tgbot::{
    longpoll::LongPoll,
    methods::SendMessage,
    types::{Command, Update, UpdateKind},
    webhook::{self, HyperError},
    Api, ApiError, Config as ApiConfig, ParseProxyError, UpdateHandler,
};
use thiserror::Error;
use tokio::sync::RwLock;

pub async fn run(config: &Config) -> Result<(), Error> {
    let mut api_config = ApiConfig::new(&config.token);

    if let Some(proxy) = &config.proxy {
        api_config = api_config.proxy(proxy)?;
    }

    let api = Api::new(api_config)?;

    if config.webhook == 0 {
        LongPoll::new(api.clone(), Handler::new(api)).run().await;
    } else {
        webhook::run_server(([0, 0, 0, 0], config.webhook), "/", Handler::new(api)).await?;
    }

    Ok(())
}

#[derive(Clone)]
struct Handler {
    api: Arc<Api>,
    command_toggle: Arc<RwLock<HashSet<i64>>>,
}

impl Handler {
    fn new(api: Api) -> Self {
        Self {
            api: Arc::new(api),
            command_toggle: Arc::new(RwLock::new(HashSet::new())),
        }
    }
}

impl UpdateHandler for Handler {
    type Future = BoxFuture<'static, ()>;

    fn handle(&self, update: Update) -> Self::Future {
        let handler = self.clone();
        Box::pin(async move {
            if let UpdateKind::Message(message) = update.kind {
                if let Some(text) = message.get_text() {
                    let chat_id = message.get_chat_id();
                    let mut result = None;

                    if text.data.starts_with('/') {
                        if let Ok(command) = Command::try_from(message) {
                            match command.get_name() {
                                "/start" => {}
                                "/dict" => {
                                    let input =
                                        command.get_message().get_text().unwrap().data[5..].trim();
                                    if !input.is_empty() {
                                        result = match bing_dict::translate(input).await {
                                            Ok(result) => Some(result.unwrap_or_else(|| {
                                                String::from("No paraphrase found")
                                            })),
                                            Err(err) => {
                                                eprintln!("{}", err);
                                                return;
                                            }
                                        };
                                    }
                                }
                                "/toggle" => {
                                    let mut command_toggle = handler.command_toggle.write().await;
                                    if command_toggle.insert(chat_id) {
                                        result = Some(String::from("OK. I will translate all non-command messages you send"));
                                    } else {
                                        command_toggle.remove(&chat_id);
                                        result = Some(String::from("OK. I will only translate the words after the /dict command"));
                                    }
                                }
                                "/help" => {}
                                "/about" => {}
                                _ => {}
                            }
                        }
                    } else {
                        let command_toggle = handler.command_toggle.read().await;
                        if command_toggle.contains(&chat_id) {
                            result = match bing_dict::translate(&text.data).await {
                                Ok(result) => Some(
                                    result.unwrap_or_else(|| String::from("No paraphrase found")),
                                ),
                                Err(err) => {
                                    eprintln!("{}", err);
                                    return;
                                }
                            }
                        }
                    }

                    if let Some(result) = result {
                        match handler.api.execute(SendMessage::new(chat_id, result)).await {
                            Ok(_) => (),
                            Err(err) => eprintln!("{}", err),
                        }
                    }
                }
            }
        })
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Api(#[from] ApiError),
    #[error(transparent)]
    Hyper(#[from] HyperError),
    #[error(transparent)]
    ParseProxy(#[from] ParseProxyError),
}
