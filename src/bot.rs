use crate::config::Config;
use futures_util::future::BoxFuture;
use tgbot::{
    longpoll::LongPoll,
    methods::SendMessage,
    types::{Command, Update, UpdateKind},
    webhook::{self, HyperError},
    Api, ApiError, Config as ApiConfig, ParseProxyError, UpdateHandler,
};
use thiserror::Error;

pub async fn run(config: &Config) -> Result<(), Error> {
    let mut api_config = ApiConfig::new(&config.token);

    if let Some(proxy) = &config.proxy {
        api_config = api_config.proxy(proxy)?;
    }

    let api = Api::new(api_config)?;

    if config.webhook == 0 {
        LongPoll::new(api.clone(), Handler::new(api, config.trigger_with_command))
            .run()
            .await;
    } else {
        webhook::run_server(
            ([0, 0, 0, 0], config.webhook),
            "/",
            Handler::new(api, config.trigger_with_command),
        )
        .await?;
    }

    Ok(())
}

#[derive(Clone)]
struct Handler {
    api: Api,
    trigger_with_command: bool,
}

impl Handler {
    fn new(api: Api, trigger_with_command: bool) -> Self {
        Self {
            api,
            trigger_with_command,
        }
    }
}

impl UpdateHandler for Handler {
    type Future = BoxFuture<'static, ()>;

    fn handle(&self, update: Update) -> Self::Future {
        let handler = self.clone();
        Box::pin(async move {
            if let UpdateKind::Message(message) = update.kind {
                let chat_id = message.get_chat_id();
                let mut input = None;

                if handler.trigger_with_command {
                    if let Ok(command) = Command::try_from(message) {
                        if command.get_name() == "/dict" {
                            input = Some(
                                command
                                    .get_args()
                                    .iter()
                                    .map(|arg| arg.as_str())
                                    .intersperse(" ")
                                    .collect::<String>(),
                            );
                        }
                    }
                } else if let Some(text) = message.get_text() {
                    input = Some(text.data.clone());
                }

                if let Some(input) = input {
                    let result = match bing_dict::translate(&input).await {
                        Ok(result) => result,
                        Err(err) => {
                            eprintln!("{}", err);
                            return;
                        }
                    };

                    let send_message;
                    if let Some(result) = result {
                        send_message = SendMessage::new(chat_id, result);
                    } else {
                        send_message = SendMessage::new(chat_id, "No paraphrase");
                    }

                    match handler.api.execute(send_message).await {
                        Ok(_) => (),
                        Err(err) => eprintln!("{}", err),
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
