use crate::config::Config;
use futures_util::future::BoxFuture;
use std::{collections::HashSet, sync::Arc};
use tgbot::{
    longpoll::LongPoll,
    methods::{GetMe, SendMessage, SetMyCommands},
    types::{BotCommand, Command, Me, MessageKind, Update, UpdateKind},
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
                    // Only handle messages that have text
                    let chat_id = message.get_chat_id();
                    let mut result = None;

                    if !text.data.starts_with('/') {
                        // The message is not a command
                        let command_toggle = context.command_toggle.read().await;

                        // Only handle the message if the chat is in the non-command-triggering list
                        if command_toggle.contains(&chat_id) {
                            let text = text.data.trim();
                            let mut input = None;

                            let mention_toggle = context.mention_toggle.read().await;

                            // Check if the chat is in the triggering-without-mention list or the chat is private
                            if mention_toggle.contains(&chat_id)
                                || (!matches!(message.kind, MessageKind::Group { .. })
                                    && !matches!(message.kind, MessageKind::Supergroup { .. }))
                            {
                                input = Some(text);
                            }

                            // Trim the argument that mentions the bot
                            if text.starts_with(&context.bot_username) {
                                input = Some(text[context.bot_username.len()..].trim_start());
                            } else if text.ends_with(&context.bot_username) {
                                input = Some(
                                    text[..text.len() - context.bot_username.len()].trim_end(),
                                );
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
                    } else if let Ok(command) = Command::try_from(message) {
                        // The Position of the argument that mentions the bot in the command
                        #[derive(PartialEq)]
                        enum ArgPos {
                            Start,
                            End,
                            None,
                        }

                        let text = &command.get_message().get_text().unwrap().data
                            [command.get_name().len()..]
                            .trim_start();

                        let mut pos = ArgPos::None;

                        // Get the argument position
                        if text.starts_with(&context.bot_username) {
                            pos = ArgPos::Start;
                        } else if text.ends_with(&context.bot_username) {
                            pos = ArgPos::End;
                        }

                        // Only handle the command if the chat is private or there is a argument that mentions the bot in the command
                        if pos != ArgPos::None
                            || (!matches!(command.get_message().kind, MessageKind::Group { .. })
                                && !matches!(
                                    command.get_message().kind,
                                    MessageKind::Supergroup { .. }
                                ))
                        {
                            match command.get_name() {
                                "/dict" => {
                                    let input;

                                    match pos {
                                        // Trim the command and the argument that mentions the bot
                                        ArgPos::Start => {
                                            input = Some(
                                                command.get_message().get_text().unwrap().data[5..]
                                                    .trim_start()
                                                    .trim_start_matches(&context.bot_username)
                                                    .trim_start(),
                                            )
                                        }
                                        ArgPos::End => {
                                            input = Some(
                                                command.get_message().get_text().unwrap().data[5..]
                                                    .trim_start()
                                                    .trim_end_matches(&context.bot_username)
                                                    .trim_end(),
                                            )
                                        }
                                        // No mentioning argument found, so this message must be sent in a private chat
                                        // Trim the command
                                        ArgPos::None => {
                                            input = Some(
                                                command.get_message().get_text().unwrap().data[5..]
                                                    .trim_start(),
                                            )
                                        }
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

                                "/toggle_command" => {
                                    let mut command_toggle = context.command_toggle.write().await;
                                    if command_toggle.insert(chat_id) {
                                        result = Some(String::from("Okay. I will translate all non-command messages you send"));
                                    } else {
                                        command_toggle.remove(&chat_id);
                                        result = Some(String::from("OK. I will only translate the words after the /dict command"));
                                    }
                                }

                                "/toggle_mention" => {
                                    // Only handle the command in group chats
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
                                        result =
                                            Some(String::from("Hmm...This is not a group chat"));
                                    }
                                }

                                "/start" => {
                                    result = Some(String::from(
                                        r#"
This Telegram bot uses Bing Dictionary to translate words from Chinese to English or English to Chinese.

/dict [word / phrase] - Translate a word or phrase
/toggle_command - Toggle translate-all-messages mode for the current chat (default: off)
/toggle_mention - Toggle if I should only react to non-command messages that mentions me in the group. You still need to @ me when using command (default: on)

Use "/help" to get more information.
"#,
                                    ));
                                }

                                "/about" => {
                                    result = Some(String::from(
                                        r#"
A Telegram bot uses Bing Dictionary to translate words and phrases from Chinese to English or English to Chinese.

https://github.com/EAimTY/bing-dict-telegram-bot
"#,
                                    ));
                                }

                                "/help" => {
                                    result = Some(String::from(
                                        r#"
/dict [word / phrase] - Translate a word or phrase
/toggle_command - Toggle translate-all-messages mode for the current chat (default: off)
/toggle_mention - Toggle if I should only react to non-command messages that mentions me in the group. You still need to @ me when using command (default: on)
/about - About this bot
/help - Get this help message
"#,
                                    ));
                                }

                                _ => {}
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
