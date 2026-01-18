use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::entities::lessons;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LessonResponse {
    pub id: Uuid,
    pub topic_id: Uuid,
    pub content: String,
}

impl From<lessons::Model> for LessonResponse {
    fn from(model: lessons::Model) -> Self {
        Self {
            id: model.id,
            topic_id: model.topic_id,
            content: model.content,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateLessonParams {
    pub topic_id: Uuid,
    #[validate(length(min = 1))]
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateLessonParams {
    pub topic_id: Option<Uuid>,
    #[validate(length(min = 1))]
    pub content: Option<String>,
}
