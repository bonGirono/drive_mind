use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::entities::categories;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CategoryResponse {
    pub id: Uuid,
    pub name: String,
}

impl From<categories::Model> for CategoryResponse {
    fn from(model: categories::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateCategoryParams {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateCategoryParams {
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,
}
