use bing_dict::Error as BingDictError;
use futures_util::future::BoxFuture;
use std::sync::Arc;
use tgbot::{
    types::{Command, Me, Update, UpdateKind},
    Api, ExecuteError, UpdateHandler,
};
use thiserror::Error;
use tinyset::Set64;
use tokio::sync::RwLock;

mod command;
mod inline_query;
mod message;

pub struct Context {
    api: Api,
    bot_username: String,
    message_trigger: RwLock<Set64<i64>>,
}

#[derive(Clone)]
pub struct Handler(Arc<Context>);

impl Handler {
    pub fn new(api: Api, bot_info: Me) -> Self {
        Self(Arc::new(Context {
            api,
            bot_username: format!("@{}", bot_info.username),
            message_trigger: RwLock::new(Set64::new()),
        }))
    }
}

impl UpdateHandler for Handler {
    type Future = BoxFuture<'static, ()>;

    fn handle(&self, update: Update) -> Self::Future {
        let context = self.0.clone();

        Box::pin(async move {
            let result: Result<(), HandlerError> = try {
                match update.kind {
                    UpdateKind::Message(message) => {
                        if let Some(text) = message.get_text() {
                            if !text.data.starts_with('/') {
                                Self::handle_message(&context, message).await?;
                            } else if let Ok(command) = Command::try_from(message) {
                                Self::handle_command(&context, command).await?;
                            }
                        }
                    }
                    UpdateKind::InlineQuery(inline_query) => {
                        Self::handle_inline_query(&context, inline_query).await?;
                    }
                    _ => {}
                }
            };

            match result {
                Ok(()) => {}
                Err(err) => eprintln!("{}", err),
            }
        })
    }
}

#[derive(Error, Debug)]
pub enum HandlerError {
    #[error(transparent)]
    Execute(#[from] ExecuteError),
    #[error(transparent)]
    BingDict(#[from] BingDictError),
}
