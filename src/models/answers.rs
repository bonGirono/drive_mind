use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

use crate::entities::answers;

#[derive(Debug, Deserialize, IntoParams)]
pub struct LangQuery {
    /// Language code (e.g., "en", "ru")
    pub lang: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AnswerResponse {
    pub id: Uuid,
    pub question_id: Uuid,
    pub value: String,
    pub is_correct: bool,
    pub lang: String,
}

impl From<answers::Model> for AnswerResponse {
    fn from(model: answers::Model) -> Self {
        Self {
            id: model.id,
            question_id: model.question_id,
            value: model.value,
            is_correct: model.is_correct,
            lang: model.lang,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateAnswerParams {
    pub question_id: Uuid,
    #[validate(length(min = 1, max = 500))]
    pub value: String,
    pub is_correct: bool,
    #[validate(length(min = 2, max = 10))]
    pub lang: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateAnswerParams {
    pub question_id: Option<Uuid>,
    #[validate(length(min = 1, max = 500))]
    pub value: Option<String>,
    pub is_correct: Option<bool>,
    #[validate(length(min = 2, max = 10))]
    pub lang: Option<String>,
}
