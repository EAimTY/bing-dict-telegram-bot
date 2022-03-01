use std::iter;

use super::Context;
use anyhow::Result;
use tgbot::{
    methods::SendMessage,
    types::{Command, ParseMode},
};

pub async fn handle_command(cx: &Context, cmd: &Command) -> Result<()> {
    let chat_id = cmd.get_message().get_chat_id();
    let msg_id = cmd.get_message().id;

    match cmd.get_name() {
        "/help" => {
            let help = r#"
/dict <i>[word / phrase]</i> - Translate a word or phrase
/toggle - Switch to the mode of translating all messages in the current chat
/about - About this bot
/help - Get this help message

Inline mode is also supported: just @ me and type in the word or phrase you want to translate
"#;
            let send_message = SendMessage::new(chat_id, help)
                .parse_mode(ParseMode::Html)
                .reply_to_message_id(msg_id);
            cx.api.execute(send_message).await?;
        }

        "/start" => {
            let start = "This bot uses Bing Dictionary to translate words and phrases between English and Chinese\n\nUse <i>/help</i> to get more information";
            let send_message = SendMessage::new(chat_id, start)
                .parse_mode(ParseMode::Html)
                .reply_to_message_id(msg_id);
            cx.api.execute(send_message).await?;
        }

        "/about" => {
            let about = "A Telegram bot using Bing Dictionary to translate words and phrases between English and Chinese\n\nhttps://github.com/EAimTY/bing-dict-telegram-bot";
            let send_message = SendMessage::new(chat_id, about).reply_to_message_id(msg_id);
            cx.api.execute(send_message).await?;
        }

        "/dict" => {
            let input = cmd
                .get_args()
                .iter()
                .filter(|arg| !arg.contains(&cx.username))
                .flat_map(|arg| iter::once(" ").chain(iter::once(arg.as_str())))
                .skip(1)
                .collect::<String>();

            let res = if !input.is_empty() {
                bing_dict::translate(&input).await?.map_or_else(
                    || String::from("No paraphrase found"),
                    |paraphrase| paraphrase.to_string(),
                )
            } else {
                String::from("No input")
            };

            let send_message = SendMessage::new(chat_id, res).reply_to_message_id(msg_id);
            cx.api.execute(send_message).await?;
        }

        "/toggle" => {
            let res = {
                let mut message_trigger = cx.message_trigger.write();

                if message_trigger.insert(chat_id) {
                    "Okay. I will translate all non-command messages in this chat (You still need to @ me if it is in a group)"
                } else {
                    message_trigger.remove(&chat_id);
                    "OK. I will only translate the words after the /dict command"
                }
            };

            cx.message_trigger.save()?;

            let send_message = SendMessage::new(chat_id, res).reply_to_message_id(msg_id);
            cx.api.execute(send_message).await?;
        }

        _ => {}
    }

    Ok(())
}
