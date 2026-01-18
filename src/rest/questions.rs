use crate::{
    AppContext,
    entities::{questions, topics},
    models::questions::{CreateQuestionParams, LangQuery, QuestionResponse, UpdateQuestionParams},
    utils::{
        extractors::{AuthUser, check_topic_access, check_topic_access_by_id},
        response::ApiError,
    },
};
use axum::{
    Json,
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter,
};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;
use validator::Validate;

/// List all questions by lang (requires auth)
#[utoipa::path(
    get,
    tag = "Questions",
    path = "/api/questions",
    params(LangQuery),
    responses(
        (status = 200, body = Vec<QuestionResponse>),
        ApiError
    ),
    security(("jwt_token" = []))
)]
async fn list(
    _auth_user: AuthUser,
    State(ctx): State<AppContext>,
    Query(query): Query<LangQuery>,
) -> axum::response::Result<Response> {
    let questions = questions::Entity::find()
        .filter(questions::Column::Lang.eq(&query.lang))
        .all(&ctx.db)
        .await
        .map_err(ApiError::from)?
        .into_iter()
        .map(QuestionResponse::from)
        .collect::<Vec<_>>();

    Ok(Json(questions).into_response())
}

/// Get question by id and lang (requires auth, subscription if topic requires)
#[utoipa::path(
    get,
    tag = "Questions",
    path = "/api/questions/{id}",
    params(
        ("id" = Uuid, Path, description = "Question ID"),
        LangQuery
    ),
    responses(
        (status = 200, body = QuestionResponse),
        ApiError
    ),
    security(("jwt_token" = []))
)]
async fn get(
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    State(ctx): State<AppContext>,
    Query(query): Query<LangQuery>,
) -> axum::response::Result<Response> {
    let question = questions::Entity::find_by_id(id)
        .filter(questions::Column::Lang.eq(&query.lang))
        .one(&ctx.db)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;

    // Проверяем доступ к topic
    check_topic_access_by_id(&ctx.db, auth_user.user.id, question.topic_id).await?;

    Ok(Json(QuestionResponse::from(question)).into_response())
}

/// Create question (requires auth, subscription if topic requires)
#[utoipa::path(
    post,
    tag = "Questions",
    path = "/api/questions",
    request_body = CreateQuestionParams,
    responses(
        (status = 201, body = QuestionResponse),
        ApiError
    ),
    security(("jwt_token" = []))
)]
async fn create(
    auth_user: AuthUser,
    State(ctx): State<AppContext>,
    Json(params): Json<CreateQuestionParams>,
) -> axum::response::Result<Response> {
    params.validate().map_err(ApiError::from)?;

    // Проверяем доступ к topic
    check_topic_access_by_id(&ctx.db, auth_user.user.id, params.topic_id).await?;

    let question = questions::ActiveModel {
        topic_id: Set(params.topic_id),
        name: Set(params.name),
        lang: Set(params.lang),
        ..Default::default()
    };

    let question = question.insert(&ctx.db).await.map_err(ApiError::from)?;

    Ok((
        axum::http::StatusCode::CREATED,
        Json(QuestionResponse::from(question)),
    )
        .into_response())
}

/// Update question by id (requires auth, subscription if topic requires)
#[utoipa::path(
    patch,
    tag = "Questions",
    path = "/api/questions/{id}",
    params(("id" = Uuid, Path, description = "Question ID")),
    request_body = UpdateQuestionParams,
    responses(
        (status = 200, body = QuestionResponse),
        ApiError
    ),
    security(("jwt_token" = []))
)]
async fn update(
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    State(ctx): State<AppContext>,
    Json(params): Json<UpdateQuestionParams>,
) -> axum::response::Result<Response> {
    params.validate().map_err(ApiError::from)?;

    let question = questions::Entity::find_by_id(id)
        .one(&ctx.db)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;

    // Проверяем доступ к текущему topic
    check_topic_access_by_id(&ctx.db, auth_user.user.id, question.topic_id).await?;

    let mut to_update = question.into_active_model();

    if let Some(topic_id) = params.topic_id {
        // Проверяем доступ к новому topic
        check_topic_access_by_id(&ctx.db, auth_user.user.id, topic_id).await?;
        to_update.topic_id = Set(topic_id);
    }
    if let Some(name) = params.name {
        to_update.name = Set(name);
    }
    if let Some(lang) = params.lang {
        to_update.lang = Set(lang);
    }

    let question = to_update.update(&ctx.db).await.map_err(ApiError::from)?;

    Ok(Json(QuestionResponse::from(question)).into_response())
}

/// Delete question by id (requires auth, subscription if topic requires)
#[utoipa::path(
    delete,
    tag = "Questions",
    path = "/api/questions/{id}",
    params(("id" = Uuid, Path, description = "Question ID")),
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
    let question = questions::Entity::find_by_id(id)
        .one(&ctx.db)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;

    // Проверяем доступ к topic
    check_topic_access_by_id(&ctx.db, auth_user.user.id, question.topic_id).await?;

    questions::Entity::delete_by_id(id)
        .exec(&ctx.db)
        .await
        .map_err(ApiError::from)?;

    Ok(().into_response())
}

/// Get questions by topic id and lang (requires auth, subscription if topic requires)
#[utoipa::path(
    get,
    tag = "Questions",
    path = "/api/topics/{topic_id}/questions",
    params(
        ("topic_id" = Uuid, Path, description = "Topic ID"),
        LangQuery
    ),
    responses(
        (status = 200, body = Vec<QuestionResponse>),
        ApiError
    ),
    security(("jwt_token" = []))
)]
async fn get_by_topic(
    auth_user: AuthUser,
    Path(topic_id): Path<Uuid>,
    State(ctx): State<AppContext>,
    Query(query): Query<LangQuery>,
) -> axum::response::Result<Response> {
    // Проверяем доступ к topic
    let topic = topics::Entity::find_by_id(topic_id)
        .one(&ctx.db)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;

    check_topic_access(&ctx.db, auth_user.user.id, &topic).await?;

    let questions = questions::Entity::find()
        .filter(questions::Column::TopicId.eq(topic_id))
        .filter(questions::Column::Lang.eq(&query.lang))
        .all(&ctx.db)
        .await
        .map_err(ApiError::from)?
        .into_iter()
        .map(QuestionResponse::from)
        .collect::<Vec<_>>();

    Ok(Json(questions).into_response())
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
