use super::Context;
use anyhow::Result;
use tgbot::{
    methods::AnswerInlineQuery,
    types::{
        InlineQuery, InlineQueryResult, InlineQueryResultArticle, InputMessageContent,
        InputMessageContentText,
    },
};

pub async fn handle_inline_query(cx: &Context, query: &InlineQuery) -> Result<()> {
    let InlineQuery { id, query, .. } = query;

    let res = if !query.is_empty() {
        let res = bing_dict::translate(query).await?.map_or_else(
            || String::from("No paraphrase found"),
            |paraphrase| paraphrase.to_string(),
        );

        vec![InlineQueryResult::Article(InlineQueryResultArticle::new(
            query,
            res.clone(),
            InputMessageContent::Text(InputMessageContentText::new(res)),
        ))]
    } else {
        Vec::new()
    };

    let answer_inline_query = AnswerInlineQuery::new(id, res);
    cx.api.execute(answer_inline_query).await?;

    Ok(())
}
