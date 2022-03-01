use anyhow::Result;
use futures_util::future::BoxFuture;
use jsave::RwLock;
use nohash_hasher::BuildNoHashHasher;
use std::{collections::HashSet, sync::Arc};
use tgbot::{
    types::{Command, MessageKind, Update, UpdateKind},
    Api, UpdateHandler,
};

mod command;
mod inline_query;
mod message;

pub struct Context {
    api: Api,
    username: String,
    message_trigger: RwLock<HashSet<i64, BuildNoHashHasher<i64>>>,
}

#[derive(Clone)]
pub struct Handler(Arc<Context>);

impl Handler {
    pub fn new(
        api: Api,
        database: RwLock<HashSet<i64, BuildNoHashHasher<i64>>>,
        username: String,
    ) -> Self {
        Self(Arc::new(Context {
            api,
            username,
            message_trigger: database,
        }))
    }
}

impl UpdateHandler for Handler {
    type Future = BoxFuture<'static, ()>;

    fn handle(&self, update: Update) -> Self::Future {
        let cx = self.0.clone();

        Box::pin(async move {
            if let Err(err) = handle_update(cx, update).await {
                eprintln!("{err}");
            }
        })
    }
}

async fn handle_update(cx: Arc<Context>, update: Update) -> Result<()> {
    match update.kind {
        UpdateKind::Message(msg) => {
            if matches!(msg.kind, MessageKind::Private { .. })
                || msg
                    .get_text()
                    .map_or(false, |text| text.data.contains(&cx.username))
            {
                if msg
                    .get_text()
                    .map_or(false, |text| text.data.starts_with('/'))
                {
                    if let Ok(cmd) = Command::try_from(msg) {
                        command::handle_command(&cx, &cmd).await?;
                    }
                } else {
                    message::handle_message(&cx, &msg).await?
                }
            }
        }
        UpdateKind::InlineQuery(query) => {
            inline_query::handle_inline_query(&cx, &query).await?;
        }
        _ => {}
    };

    Ok(())
}
