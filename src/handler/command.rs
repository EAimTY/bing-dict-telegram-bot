use super::{Context, Handler, HandlerError};
use tgbot::{
    methods::SendMessage,
    types::{Command, MessageKind},
};

impl Handler {
    pub async fn handle_command(context: &Context, command: Command) -> Result<(), HandlerError> {
        let chat_id = command.get_message().get_chat_id();

        // The Position of the argument that mentions the bot in the command
        #[derive(PartialEq)]
        enum ArgPos {
            Start,
            End,
            None,
        }

        let text = &command.get_message().get_text().unwrap().data[command.get_name().len()..]
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
                && !matches!(command.get_message().kind, MessageKind::Supergroup { .. }))
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

                "/toggle_command" => {
                    let mut command_toggle = context.command_toggle.write().await;
                    let result = if command_toggle.insert(chat_id) {
                        String::from("Okay. I will translate all non-command messages you send")
                    } else {
                        command_toggle.remove(&chat_id);
                        String::from("OK. I will only translate the words after the /dict command")
                    };
                    context
                        .api
                        .execute(SendMessage::new(chat_id, result))
                        .await?;
                }

                "/toggle_mention" => {
                    // Only handle the command in group chats
                    let result = if matches!(command.get_message().kind, MessageKind::Group { .. })
                        || matches!(command.get_message().kind, MessageKind::Supergroup { .. })
                    {
                        let mut mention_toggle = context.mention_toggle.write().await;
                        if mention_toggle.insert(chat_id) {
                            String::from("Okay. Now you don't need to @ me to trigger a non-command message translation anymore")
                        } else {
                            mention_toggle.remove(&chat_id);
                            String::from(
                                "Fine. I will only react to non-command messages that mentioned me",
                            )
                        }
                    } else {
                        String::from("Hmm...This is not a group chat")
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
/toggle_command - Toggle translate-all-messages mode for the current chat (default: off)
/toggle_mention - Toggle if I should only react to non-command messages that mentions me in the group. You still need to @ me when using command (default: on)

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
/toggle_command - Toggle translate-all-messages mode for the current chat (default: off)
/toggle_mention - Toggle if I should only react to non-command messages that mentions me in the group. You still need to @ me when using command (default: on)
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