use super::{Context, Handler, HandlerError};
use tgbot::{
    methods::AnswerInlineQuery,
    types::{
        InlineQuery, InlineQueryResult, InlineQueryResultArticle, InputMessageContent,
        InputMessageContentText,
    },
};

impl Handler {
    pub async fn handle_inline_query(
        context: &Context,
        inline_query: InlineQuery,
    ) -> Result<(), HandlerError> {
        let InlineQuery { id, query, .. } = inline_query;

        let inline_query_result = if !query.is_empty() {
            let translate_result = bing_dict::translate(&query)
                .await?
                .unwrap_or_else(|| String::from("No paraphrase found"));

            vec![InlineQueryResult::Article(InlineQueryResultArticle::new(
                query,
                translate_result.clone(),
                InputMessageContent::Text(InputMessageContentText::new(translate_result)),
            ))]
        } else {
            Vec::new()
        };

        context
            .api
            .execute(AnswerInlineQuery::new(id, inline_query_result))
            .await?;

        Ok(())
    }
}
