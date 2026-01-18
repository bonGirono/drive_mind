use crate::{
    AppContext,
    entities::{lessons, topics},
    models::lessons::{CreateLessonParams, LessonResponse, UpdateLessonParams},
    utils::{
        extractors::{AuthUser, check_topic_access, check_topic_access_by_id},
        response::ApiError,
    },
};
use axum::{
    Json,
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter,
};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;
use validator::Validate;

/// List all lessons (requires auth, subscription check per topic)
#[utoipa::path(
    get,
    tag = "Lessons",
    path = "/api/lessons",
    responses(
        (status = 200, body = Vec<LessonResponse>),
        ApiError
    ),
    security(("jwt_token" = []))
)]
async fn list(
    _auth_user: AuthUser,
    State(ctx): State<AppContext>,
) -> axum::response::Result<Response> {
    let lessons = lessons::Entity::find()
        .all(&ctx.db)
        .await
        .map_err(ApiError::from)?
        .into_iter()
        .map(LessonResponse::from)
        .collect::<Vec<_>>();

    Ok(Json(lessons).into_response())
}

/// Get lesson by id (requires auth, subscription if topic requires)
#[utoipa::path(
    get,
    tag = "Lessons",
    path = "/api/lessons/{id}",
    params(("id" = Uuid, Path, description = "Lesson ID")),
    responses(
        (status = 200, body = LessonResponse),
        ApiError
    ),
    security(("jwt_token" = []))
)]
async fn get(
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    State(ctx): State<AppContext>,
) -> axum::response::Result<Response> {
    let lesson = lessons::Entity::find_by_id(id)
        .one(&ctx.db)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;

    // Проверяем доступ к topic
    check_topic_access_by_id(&ctx.db, auth_user.user.id, lesson.topic_id).await?;

    Ok(Json(LessonResponse::from(lesson)).into_response())
}

/// Create lesson (requires auth, subscription if topic requires)
#[utoipa::path(
    post,
    tag = "Lessons",
    path = "/api/lessons",
    request_body = CreateLessonParams,
    responses(
        (status = 201, body = LessonResponse),
        ApiError
    ),
    security(("jwt_token" = []))
)]
async fn create(
    auth_user: AuthUser,
    State(ctx): State<AppContext>,
    Json(params): Json<CreateLessonParams>,
) -> axum::response::Result<Response> {
    params.validate().map_err(ApiError::from)?;

    // Проверяем доступ к topic
    check_topic_access_by_id(&ctx.db, auth_user.user.id, params.topic_id).await?;

    let lesson = lessons::ActiveModel {
        topic_id: Set(params.topic_id),
        content: Set(params.content),
        ..Default::default()
    };

    let lesson = lesson.insert(&ctx.db).await.map_err(ApiError::from)?;

    Ok((
        axum::http::StatusCode::CREATED,
        Json(LessonResponse::from(lesson)),
    )
        .into_response())
}

/// Update lesson by id (requires auth, subscription if topic requires)
#[utoipa::path(
    patch,
    tag = "Lessons",
    path = "/api/lessons/{id}",
    params(("id" = Uuid, Path, description = "Lesson ID")),
    request_body = UpdateLessonParams,
    responses(
        (status = 200, body = LessonResponse),
        ApiError
    ),
    security(("jwt_token" = []))
)]
async fn update(
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    State(ctx): State<AppContext>,
    Json(params): Json<UpdateLessonParams>,
) -> axum::response::Result<Response> {
    params.validate().map_err(ApiError::from)?;

    let lesson = lessons::Entity::find_by_id(id)
        .one(&ctx.db)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;

    // Проверяем доступ к текущему topic
    check_topic_access_by_id(&ctx.db, auth_user.user.id, lesson.topic_id).await?;

    let mut to_update = lesson.into_active_model();

    if let Some(topic_id) = params.topic_id {
        // Проверяем доступ к новому topic
        check_topic_access_by_id(&ctx.db, auth_user.user.id, topic_id).await?;
        to_update.topic_id = Set(topic_id);
    }
    if let Some(content) = params.content {
        to_update.content = Set(content);
    }

    let lesson = to_update.update(&ctx.db).await.map_err(ApiError::from)?;

    Ok(Json(LessonResponse::from(lesson)).into_response())
}

/// Delete lesson by id (requires auth, subscription if topic requires)
#[utoipa::path(
    delete,
    tag = "Lessons",
    path = "/api/lessons/{id}",
    params(("id" = Uuid, Path, description = "Lesson ID")),
    responses(
        (status = 200),
        ApiError
    ),
    security(("jwt_token" = []))
)]
async fn delete(
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    State(ctx): State<AppContext>,
) -> axum::response::Result<Response> {
    let lesson = lessons::Entity::find_by_id(id)
        .one(&ctx.db)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;

    // Проверяем доступ к topic
    check_topic_access_by_id(&ctx.db, auth_user.user.id, lesson.topic_id).await?;

    lessons::Entity::delete_by_id(id)
        .exec(&ctx.db)
        .await
        .map_err(ApiError::from)?;

    Ok(().into_response())
}

/// Get lesson by topic id (requires auth, subscription if topic requires)
#[utoipa::path(
    get,
    tag = "Lessons",
    path = "/api/topics/{topic_id}/lesson",
    params(("topic_id" = Uuid, Path, description = "Topic ID")),
    responses(
        (status = 200, body = LessonResponse),
        ApiError
    ),
    security(("jwt_token" = []))
)]
async fn get_by_topic(
    auth_user: AuthUser,
    Path(topic_id): Path<Uuid>,
    State(ctx): State<AppContext>,
) -> axum::response::Result<Response> {
    // Проверяем доступ к topic
    let topic = topics::Entity::find_by_id(topic_id)
        .one(&ctx.db)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;

    check_topic_access(&ctx.db, auth_user.user.id, &topic).await?;

    let lesson = lessons::Entity::find()
        .filter(lessons::Column::TopicId.eq(topic_id))
        .one(&ctx.db)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;

    Ok(Json(LessonResponse::from(lesson)).into_response())
}

pub fn routes() -> OpenApiRouter<AppContext> {
    OpenApiRouter::new()
        .routes(routes!(list))
        .routes(routes!(get))
        .routes(routes!(create))
        .routes(routes!(update))
        .routes(routes!(delete))
        .routes(routes!(get_by_topic))
}
