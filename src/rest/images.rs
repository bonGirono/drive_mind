use crate::{
    AppContext,
    entities::images,
    models::images::{ImageResponse, UploadResponse},
    utils::response::ApiError,
};
use axum::{
    Json,
    body::Body,
    extract::{Multipart, Path, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait};
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio_util::io::ReaderStream;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

const UPLOAD_DIR: &str = "uploads/images";

/// Upload image
#[utoipa::path(
    post,
    tag = "Images",
    path = "/api/images",
    request_body(content_type = "multipart/form-data", content = Vec<u8>),
    responses(
        (status = 201, body = UploadResponse),
        ApiError
    ),
    security()
)]
async fn upload(
    State(ctx): State<AppContext>,
    mut multipart: Multipart,
) -> axum::response::Result<Response> {
    fs::create_dir_all(UPLOAD_DIR).await.map_err(|e| {
        tracing::error!("Failed to create upload dir: {}", e);
        ApiError::InternalServerError
    })?;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        tracing::error!("Failed to read multipart field: {}", e);
        ApiError::BadRequest
    })? {
        let content_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();

        if !content_type.starts_with("image/") {
            return Err(ApiError::InvalidFormat.into());
        }

        let original_name = field.file_name().unwrap_or("unknown").to_string();
        let extension = original_name.rsplit('.').next().unwrap_or("bin");

        let id = Uuid::new_v4();
        let stored_name = format!("{}.{}", id, extension);
        let file_path = PathBuf::from(UPLOAD_DIR).join(&stored_name);

        let data = field.bytes().await.map_err(|e| {
            tracing::error!("Failed to read file bytes: {}", e);
            ApiError::BadRequest
        })?;

        let size = data.len() as i64;

        let mut file = fs::File::create(&file_path).await.map_err(|e| {
            tracing::error!("Failed to create file: {}", e);
            ApiError::InternalServerError
        })?;

        file.write_all(&data).await.map_err(|e| {
            tracing::error!("Failed to write file: {}", e);
            ApiError::InternalServerError
        })?;

        let image = images::ActiveModel {
            id: Set(id),
            original_name: Set(original_name),
            stored_name: Set(stored_name.clone()),
            mime_type: Set(content_type),
            size: Set(size),
            ..Default::default()
        };

        image.insert(&ctx.db).await.map_err(ApiError::from)?;

        let response = UploadResponse {
            id,
            stored_name: stored_name.clone(),
            url: format!("/api/images/file/{}", stored_name),
        };

        return Ok((StatusCode::CREATED, Json(response)).into_response());
    }

    Err(ApiError::MissingField.into())
}

/// Get image file by filename
#[utoipa::path(
    get,
    tag = "Images",
    path = "/api/images/file/{filename}",
    params(("filename" = String, Path, description = "Image filename")),
    responses(
        (status = 200, content_type = "image/*"),
        ApiError
    ),
    security()
)]
async fn get_image(Path(filename): Path<String>) -> axum::response::Result<Response> {
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err(ApiError::BadRequest.into());
    }

    let file_path = PathBuf::from(UPLOAD_DIR).join(&filename);

    let file = fs::File::open(&file_path)
        .await
        .map_err(|_| ApiError::NotFound)?;

    let content_type = match filename.rsplit('.').next() {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("bmp") => "image/bmp",
        _ => "application/octet-stream",
    };

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, "public, max-age=31536000")
        .body(body)
        .unwrap())
}

/// List all images
#[utoipa::path(
    get,
    tag = "Images",
    path = "/api/images",
    responses(
        (status = 200, body = Vec<ImageResponse>),
        ApiError
    ),
    security()
)]
async fn list(State(ctx): State<AppContext>) -> axum::response::Result<Response> {
    let images = images::Entity::find()
        .all(&ctx.db)
        .await
        .map_err(ApiError::from)?
        .into_iter()
        .map(ImageResponse::from)
        .collect::<Vec<_>>();

    Ok(Json(images).into_response())
}

/// Delete image by id
#[utoipa::path(
    delete,
    tag = "Images",
    path = "/api/images/{id}",
    params(("id" = Uuid, Path, description = "Image ID")),
    responses(
        (status = 200),
        ApiError
    ),
    security()
)]
async fn delete(
    Path(id): Path<Uuid>,
    State(ctx): State<AppContext>,
) -> axum::response::Result<Response> {
    let image = images::Entity::find_by_id(id)
        .one(&ctx.db)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;

    let file_path = PathBuf::from(UPLOAD_DIR).join(&image.stored_name);
    if file_path.exists() {
        fs::remove_file(&file_path).await.map_err(|e| {
            tracing::error!("Failed to delete file: {}", e);
            ApiError::InternalServerError
        })?;
    }

    images::Entity::delete_by_id(id)
        .exec(&ctx.db)
        .await
        .map_err(ApiError::from)?;

    Ok(().into_response())
}

pub fn routes() -> OpenApiRouter<AppContext> {
    OpenApiRouter::new()
        .routes(routes!(upload))
        .routes(routes!(list))
        .routes(routes!(get_image))
        .routes(routes!(delete))
}
