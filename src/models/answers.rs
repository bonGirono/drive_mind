use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::entities::answers;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AnswerResponse {
    pub id: Uuid,
    pub question_id: Uuid,
    pub value: String,
    pub is_correct: bool,
}

impl From<answers::Model> for AnswerResponse {
    fn from(model: answers::Model) -> Self {
        Self {
            id: model.id,
            question_id: model.question_id,
            value: model.value,
            is_correct: model.is_correct,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateAnswerParams {
    pub question_id: Uuid,
    #[validate(length(min = 1, max = 500))]
    pub value: String,
    pub is_correct: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateAnswerParams {
    pub question_id: Option<Uuid>,
    #[validate(length(min = 1, max = 500))]
    pub value: Option<String>,
    pub is_correct: Option<bool>,
}
