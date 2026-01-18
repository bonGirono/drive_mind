use crate::{
    AppContext,
    entities::topics,
    models::topics::{CreateTopicParams, TopicResponse, UpdateTopicParams},
    utils::response::ApiError,
};
use axum::{
    Json,
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait, IntoActiveModel};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;
use validator::Validate;

/// List all topics
#[utoipa::path(
    get,
    tag = "Topics",
    path = "/api/topics",
    responses(
        (status = 200, body = Vec<TopicResponse>),
        ApiError
    ),
    security()
)]
async fn list(State(ctx): State<AppContext>) -> axum::response::Result<Response> {
    let topics = topics::Entity::find()
        .all(&ctx.db)
        .await
        .map_err(ApiError::from)?
        .into_iter()
        .map(TopicResponse::from)
        .collect::<Vec<_>>();

    Ok(Json(topics).into_response())
}

/// Get topic by id
#[utoipa::path(
    get,
    tag = "Topics",
    path = "/api/topics/{id}",
    params(("id" = Uuid, Path, description = "Topic ID")),
    responses(
        (status = 200, body = TopicResponse),
        ApiError
    ),
    security()
)]
async fn get(
    Path(id): Path<Uuid>,
    State(ctx): State<AppContext>,
) -> axum::response::Result<Response> {
    let topic = topics::Entity::find_by_id(id)
        .one(&ctx.db)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;

    Ok(Json(TopicResponse::from(topic)).into_response())
}

/// Create topic
#[utoipa::path(
    post,
    tag = "Topics",
    path = "/api/topics",
    request_body = CreateTopicParams,
    responses(
        (status = 201, body = TopicResponse),
        ApiError
    ),
    security()
)]
async fn create(
    State(ctx): State<AppContext>,
    Json(params): Json<CreateTopicParams>,
) -> axum::response::Result<Response> {
    params.validate().map_err(ApiError::from)?;

    let topic = topics::ActiveModel {
        name: Set(params.name),
        difficulty: Set(params.difficulty),
        duration: Set(params.duration),
        subscription_required: Set(params.subscription_required),
        ..Default::default()
    };

    let topic = topic.insert(&ctx.db).await.map_err(ApiError::from)?;

    Ok((
        axum::http::StatusCode::CREATED,
        Json(TopicResponse::from(topic)),
    )
        .into_response())
}

/// Update topic by id
#[utoipa::path(
    patch,
    tag = "Topics",
    path = "/api/topics/{id}",
    params(("id" = Uuid, Path, description = "Topic ID")),
    request_body = UpdateTopicParams,
    responses(
        (status = 200, body = TopicResponse),
        ApiError
    ),
    security()
)]
async fn update(
    Path(id): Path<Uuid>,
    State(ctx): State<AppContext>,
    Json(params): Json<UpdateTopicParams>,
) -> axum::response::Result<Response> {
    params.validate().map_err(ApiError::from)?;

    let topic = topics::Entity::find_by_id(id)
        .one(&ctx.db)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;

    let mut to_update = topic.into_active_model();

    if let Some(name) = params.name {
        to_update.name = Set(name);
    }
    if let Some(difficulty) = params.difficulty {
        to_update.difficulty = Set(difficulty);
    }
    if let Some(duration) = params.duration {
        to_update.duration = Set(duration);
    }
    if let Some(subscription_required) = params.subscription_required {
        to_update.subscription_required = Set(subscription_required);
    }

    let topic = to_update.update(&ctx.db).await.map_err(ApiError::from)?;

    Ok(Json(TopicResponse::from(topic)).into_response())
}

/// Delete topic by id
#[utoipa::path(
    delete,
    tag = "Topics",
    path = "/api/topics/{id}",
    params(("id" = Uuid, Path, description = "Topic ID")),
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
    topics::Entity::delete_by_id(id)
        .exec(&ctx.db)
        .await
        .map_err(ApiError::from)?;

    Ok(().into_response())
}

pub fn routes() -> OpenApiRouter<AppContext> {
    OpenApiRouter::new()
        .routes(routes!(list))
        .routes(routes!(get))
        .routes(routes!(create))
        .routes(routes!(update))
        .routes(routes!(delete))
}
