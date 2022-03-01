use super::Context;
use anyhow::Result;
use tgbot::{methods::SendMessage, types::Message};

pub async fn handle_message(cx: &Context, msg: &Message) -> Result<()> {
    if let Some(text) = msg.get_text() {
        let chat_id = msg.get_chat_id();
        let msg_id = msg.id;

        if cx.message_trigger.read().contains(&chat_id) {
            let input = text.data.replace(&cx.username, "");

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
    }

    Ok(())
}
