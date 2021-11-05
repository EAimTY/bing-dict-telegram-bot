use super::{Context, Handler, HandlerError};
use tgbot::{
    methods::SendMessage,
    types::{Command, MessageKind, ParseMode},
};

impl Handler {
    pub async fn handle_command(context: &Context, command: Command) -> Result<(), HandlerError> {
        let chat_id = command.get_message().get_chat_id();
        let message_id = command.get_message().id;

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
                            .execute(
                                SendMessage::new(chat_id, result).reply_to_message_id(message_id),
                            )
                            .await?;
                    }
                }

                "/toggle" => {
                    let mut message_trigger = context.message_trigger.write().await;
                    let result = if message_trigger.insert(chat_id) {
                        format!("Okay. I will translate all non-command messages you send (You still need to <i>{}</i> if it is in a group)", context.bot_username)
                    } else {
                        message_trigger.remove(&chat_id);
                        String::from("OK. I will only translate the words after the /dict command")
                    };

                    context
                        .api
                        .execute(
                            SendMessage::new(chat_id, result)
                                .parse_mode(ParseMode::Html)
                                .reply_to_message_id(message_id),
                        )
                        .await?;
                }

                "/start" => {
                    let result = format!(
                        r#"
This Telegram bot uses Bing Dictionary to translate words and phrases from Chinese to English or English to Chinese.

/dict <i>[word / phrase]</i> - Translate a word or phrase
/toggle - Switch to the mode of translating all messages in the current chat (<i>{}</i> is required if it is a group)

Or just enter <i>{} word / phrase</i> in any chat and select the result when you need a translate

Use <i>/help</i> to get more information.
"#,
                        context.bot_username, context.bot_username
                    );

                    context
                        .api
                        .execute(
                            SendMessage::new(chat_id, result)
                                .parse_mode(ParseMode::Html)
                                .reply_to_message_id(message_id),
                        )
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
                        .execute(SendMessage::new(chat_id, result).reply_to_message_id(message_id))
                        .await?;
                }

                "/help" => {
                    let result = format!(
                        r#"
/dict <i>[word / phrase]</i> - Translate a word or phrase
/toggle - Switch to the mode of translating all messages in the current chat (<i>{}</i> is required if it is a group)
/about - About this bot
/help - Get this help message

When you need a translate, just enter <i>{} word / phrase</i> in any chat and select the result 
"#,
                        context.bot_username, context.bot_username
                    );

                    context
                        .api
                        .execute(
                            SendMessage::new(chat_id, result)
                                .parse_mode(ParseMode::Html)
                                .reply_to_message_id(message_id),
                        )
                        .await?;
                }

                _ => {}
            }
        }

        Ok(())
    }
}
