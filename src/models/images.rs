use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::entities::images;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ImageResponse {
    pub id: Uuid,
    pub original_name: String,
    pub stored_name: String,
    pub mime_type: String,
    pub size: i64,
}

impl From<images::Model> for ImageResponse {
    fn from(value: images::Model) -> Self {
        Self {
            id: value.id,
            original_name: value.original_name,
            stored_name: value.stored_name,
            mime_type: value.mime_type,
            size: value.size,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UploadResponse {
    pub id: Uuid,
    pub stored_name: String,
    pub url: String,
}
