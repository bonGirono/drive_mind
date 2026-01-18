use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::entities::topics;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TopicResponse {
    pub id: Uuid,
    pub name: String,
    pub difficulty: String,
    pub duration: i16,
    pub subscription_required: bool,
}

impl From<topics::Model> for TopicResponse {
    fn from(model: topics::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            difficulty: model.difficulty,
            duration: model.duration,
            subscription_required: model.subscription_required,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateTopicParams {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[validate(length(min = 1, max = 50))]
    pub difficulty: String,
    pub duration: i16,
    #[serde(default)]
    pub subscription_required: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateTopicParams {
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,
    #[validate(length(min = 1, max = 50))]
    pub difficulty: Option<String>,
    pub duration: Option<i16>,
    pub subscription_required: Option<bool>,
}
