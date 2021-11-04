use super::{Context, Handler, HandlerError};
use tgbot::{
    methods::SendMessage,
    types::{Command, MessageKind},
};

impl Handler {
    pub async fn handle_command(context: &Context, command: Command) -> Result<(), HandlerError> {
        let chat_id = command.get_message().get_chat_id();

        #[derive(PartialEq)]
        enum ArgPos {
            Start,
            End,
            None,
        }

        let text = &command.get_message().get_text().unwrap().data[command.get_name().len()..]
            .trim_start();

        let mut pos = ArgPos::None;

        if text.starts_with(&context.bot_username) {
            pos = ArgPos::Start;
        } else if text.ends_with(&context.bot_username) {
            pos = ArgPos::End;
        }

        if pos != ArgPos::None
            || (!matches!(command.get_message().kind, MessageKind::Group { .. })
                && !matches!(command.get_message().kind, MessageKind::Supergroup { .. }))
        {
            match command.get_name() {
                "/dict" => {
                    let input;

                    match pos {
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
                        ArgPos::None => {
                            input = Some(
                                command.get_message().get_text().unwrap().data[5..].trim_start(),
                            )
                        }
                    }

                    if let Some(input) = input {
                        let result = if !input.is_empty() {
                            bing_dict::translate(input)
                                .await?
                                .unwrap_or_else(|| String::from("No paraphrase found"))
                        } else {
                            String::from("No input")
                        };

                        context
                            .api
                            .execute(SendMessage::new(chat_id, result))
                            .await?;
                    }
                }

                "/toggle" => {
                    let mut message_trigger = context.message_trigger.write().await;
                    let result = if message_trigger.insert(chat_id) {
                        String::from("Okay. I will translate all non-command messages you send (You still need to @ me if it is in a group)")
                    } else {
                        message_trigger.remove(&chat_id);
                        String::from("OK. I will only translate the words after the /dict command")
                    };
                    context
                        .api
                        .execute(SendMessage::new(chat_id, result))
                        .await?;
                }

                "/start" => {
                    let result = String::from(
                        r#"
This Telegram bot uses Bing Dictionary to translate words and phrases from Chinese to English or English to Chinese.

/dict [word / phrase] - Translate a word or phrase
/toggle - Switch to the mode of translating all messages in the current chat (@ me is required if it is a group)

Use "/help" to get more information.
"#,
                    );
                    context
                        .api
                        .execute(SendMessage::new(chat_id, result))
                        .await?;
                }

                "/about" => {
                    let result = String::from(
                        r#"
A Telegram bot uses Bing Dictionary to translate words and phrases from Chinese to English or English to Chinese.

https://github.com/EAimTY/bing-dict-telegram-bot
"#,
                    );
                    context
                        .api
                        .execute(SendMessage::new(chat_id, result))
                        .await?;
                }

                "/help" => {
                    let result = String::from(
                        r#"
/dict [word / phrase] - Translate a word or phrase
/toggle - Switch to the mode of translating all messages in the current chat (@ me is required if it is a group)
/about - About this bot
/help - Get this help message
"#,
                    );
                    context
                        .api
                        .execute(SendMessage::new(chat_id, result))
                        .await?;
                }

                _ => {}
            }
        }

        Ok(())
    }
}
