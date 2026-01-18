use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct QuestionCategoryResponse {
    pub question_id: Uuid,
    pub category_id: Uuid,
}
