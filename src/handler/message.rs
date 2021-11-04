use super::{Context, Handler, HandlerError};
use tgbot::{
    methods::SendMessage,
    types::{Message, MessageKind},
};

impl Handler {
    pub async fn handle_message(context: &Context, message: Message) -> Result<(), HandlerError> {
        let chat_id = message.get_chat_id();

        let command_toggle = context.command_toggle.read().await;

        // Only handle the message if the chat is in the non-command-triggering list
        if command_toggle.contains(&chat_id) {
            let text = message.get_text().unwrap().data.trim();
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
                input = Some(text[..text.len() - context.bot_username.len()].trim_end());
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

        Ok(())
    }
}
