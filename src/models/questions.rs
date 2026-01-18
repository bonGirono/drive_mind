use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

use crate::entities::questions;

#[derive(Debug, Deserialize, IntoParams)]
pub struct LangQuery {
    /// Language code (e.g., "en", "ru")
    pub lang: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct QuestionResponse {
    pub id: Uuid,
    pub topic_id: Uuid,
    pub name: String,
    pub lang: String,
    pub content: Option<String>,
    pub explanation: String,
}

impl From<questions::Model> for QuestionResponse {
    fn from(model: questions::Model) -> Self {
        Self {
            id: model.id,
            topic_id: model.topic_id,
            name: model.name,
            lang: model.lang,
            content: model.content,
            explanation: model.explanation,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateQuestionParams {
    pub topic_id: Uuid,
    #[validate(length(min = 1, max = 500))]
    pub name: String,
    #[validate(length(min = 2, max = 10))]
    pub lang: String,
    pub content: Option<String>,
    #[validate(length(min = 1))]
    pub explanation: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateQuestionParams {
    pub topic_id: Option<Uuid>,
    #[validate(length(min = 1, max = 500))]
    pub name: Option<String>,
    #[validate(length(min = 2, max = 10))]
    pub lang: Option<String>,
    pub content: Option<String>,
    #[validate(length(min = 1))]
    pub explanation: Option<String>,
}
