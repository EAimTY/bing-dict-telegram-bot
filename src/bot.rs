use crate::config::Config;
use futures_util::future::BoxFuture;
use std::{collections::HashSet, sync::Arc};
use tgbot::{
    longpoll::LongPoll,
    methods::{GetMe, SendMessage},
    types::{Command, Me, MessageKind, Update, UpdateKind},
    webhook::{self, HyperError},
    Api, ApiError, Config as ApiConfig, ExecuteError, ParseProxyError, UpdateHandler,
};
use thiserror::Error;
use tokio::sync::RwLock;

pub async fn run(config: &Config) -> Result<(), Error> {
    let mut api_config = ApiConfig::new(&config.token);

    if let Some(proxy) = &config.proxy {
        api_config = api_config.proxy(proxy)?;
    }

    let api = Api::new(api_config)?;

    let bot_info = api.execute(GetMe).await?;

    if config.webhook == 0 {
        LongPoll::new(api.clone(), Handler::new(api, bot_info))
            .run()
            .await;
    } else {
        webhook::run_server(
            ([0, 0, 0, 0], config.webhook),
            "/",
            Handler::new(api, bot_info),
        )
        .await?;
    }

    Ok(())
}

struct Context {
    api: Api,
    bot_username: String,
    command_toggle: RwLock<HashSet<i64>>,
    mention_toggle: RwLock<HashSet<i64>>,
}

#[derive(Clone)]
struct Handler(Arc<Context>);

impl Handler {
    fn new(api: Api, bot_info: Me) -> Self {
        Self(Arc::new(Context {
            api,
            bot_username: format!("@{}", bot_info.username),
            command_toggle: RwLock::new(HashSet::new()),
            mention_toggle: RwLock::new(HashSet::new()),
        }))
    }
}

impl UpdateHandler for Handler {
    type Future = BoxFuture<'static, ()>;

    fn handle(&self, update: Update) -> Self::Future {
        let context = self.0.clone();

        Box::pin(async move {
            if let UpdateKind::Message(message) = update.kind {
                if let Some(text) = message.get_text() {
                    let chat_id = message.get_chat_id();
                    let mut result = None;

                    if !text.data.starts_with('/') {
                        let command_toggle = context.command_toggle.read().await;
                        if command_toggle.contains(&chat_id) {
                            let text = text.data.trim();
                            let mut input = None;

                            let mention_toggle = context.mention_toggle.read().await;
                            if mention_toggle.contains(&chat_id)
                                || (!matches!(message.kind, MessageKind::Group { .. })
                                    && !matches!(message.kind, MessageKind::Supergroup { .. }))
                            {
                                input = Some(text);
                            }

                            if text.starts_with(&context.bot_username) {
                                input = Some(text[context.bot_username.len()..].trim());
                            } else if text.ends_with(&context.bot_username) {
                                input =
                                    Some(text[..text.len() - context.bot_username.len()].trim());
                            }

                            if let Some(input) = input {
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
                                } else {
                                    result = Some(String::from("No input"));
                                }
                            }
                        }
                    } else {
                        if let Ok(command) = Command::try_from(message) {
                            #[derive(PartialEq)]
                            enum ArgPos {
                                Left,
                                Right,
                                None,
                            }

                            let mut pos = ArgPos::None;

                            if command.get_args().first() == Some(&context.bot_username) {
                                pos = ArgPos::Left;
                            } else if command.get_args().last() == Some(&context.bot_username) {
                                pos = ArgPos::Right;
                            }

                            if pos != ArgPos::None
                                || (!matches!(
                                    command.get_message().kind,
                                    MessageKind::Group { .. }
                                ) && !matches!(
                                    command.get_message().kind,
                                    MessageKind::Supergroup { .. }
                                ))
                            {
                                match command.get_name() {
                                    "/dict" => {
                                        let input;

                                        match pos {
                                            ArgPos::Left => {
                                                input = Some(
                                                    command.get_message().get_text().unwrap().data
                                                        [5..]
                                                        .trim()
                                                        .trim_start_matches(&context.bot_username)
                                                        .trim(),
                                                )
                                            }
                                            ArgPos::Right => {
                                                input = Some(
                                                    command.get_message().get_text().unwrap().data
                                                        [5..]
                                                        .trim()
                                                        .trim_end_matches(&context.bot_username)
                                                        .trim(),
                                                )
                                            }
                                            ArgPos::None => {
                                                input = Some(
                                                    command.get_message().get_text().unwrap().data
                                                        [5..]
                                                        .trim(),
                                                )
                                            }
                                        }

                                        if let Some(input) = input {
                                            if !input.is_empty() {
                                                result = match bing_dict::translate(input).await {
                                                    Ok(result) => {
                                                        Some(result.unwrap_or_else(|| {
                                                            String::from("No paraphrase found")
                                                        }))
                                                    }
                                                    Err(err) => {
                                                        eprintln!("{}", err);
                                                        return;
                                                    }
                                                };
                                            } else {
                                                result = Some(String::from("No input"));
                                            }
                                        }
                                    }

                                    "/toggle_command" => {
                                        let mut command_toggle =
                                            context.command_toggle.write().await;
                                        if command_toggle.insert(chat_id) {
                                            result = Some(String::from("Okay. I will translate all non-command messages you send"));
                                        } else {
                                            command_toggle.remove(&chat_id);
                                            result = Some(String::from("OK. I will only translate the words after the /dict command"));
                                        }
                                    }

                                    "/toggle_mention" => {
                                        if matches!(
                                            command.get_message().kind,
                                            MessageKind::Group { .. }
                                        ) || matches!(
                                            command.get_message().kind,
                                            MessageKind::Supergroup { .. }
                                        ) {
                                            let mut mention_toggle =
                                                context.mention_toggle.write().await;
                                            if mention_toggle.insert(chat_id) {
                                                result = Some(String::from("Okay. Now you don't need to @ me to trigger a non-command message translation anymore"));
                                            } else {
                                                mention_toggle.remove(&chat_id);
                                                result = Some(String::from("Fine. I will only react to non-command messages that mentioned me"));
                                            }
                                        } else {
                                            result = Some(String::from("This is not a group chat"));
                                        }
                                    }

                                    "/start" => {
                                        result = Some(String::from(
                                            r#"
This is a Telegram bot uses Bing Dictionary to translate words from Chinese to English or English to Chinese.

/dict [word] - translate a word
/toggle - toggle translate-all-messages mode for current chat

Use "/help" to get more information.
"#,
                                        ));
                                    }

                                    "/about" => {
                                        result = Some(String::from(
                                            r#"
A Telegram bot uses Bing Dictionary to translate words from Chinese to English or English to Chinese.

https://github.com/EAimTY/bing-dict-telegram-bot
"#,
                                        ));
                                    }

                                    "/help" => {
                                        result = Some(String::from(
                                            r#"
/dict [word] - translate a word
/toggle - toggle translate-all-messages mode for current chat
/about - About this bot
/help - Get this help message
"#,
                                        ));
                                    }

                                    _ => {}
                                }
                            }
                        }
                    }

                    if let Some(result) = result {
                        match context.api.execute(SendMessage::new(chat_id, result)).await {
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
    Execute(#[from] ExecuteError),
    #[error(transparent)]
    Hyper(#[from] HyperError),
    #[error(transparent)]
    ParseProxy(#[from] ParseProxyError),
}
