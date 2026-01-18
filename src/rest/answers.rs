use crate::{
    AppContext,
    entities::{answers, questions},
    models::answers::{AnswerResponse, CreateAnswerParams, UpdateAnswerParams},
    utils::{
        extractors::{AuthUser, check_topic_access_by_id},
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

/// Получает topic_id по question_id
async fn get_topic_id_by_question(
    db: &sea_orm::DatabaseConnection,
    question_id: Uuid,
) -> Result<Uuid, ApiError> {
    let question = questions::Entity::find_by_id(question_id)
        .one(db)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;

    Ok(question.topic_id)
}

/// List all answers (requires auth)
#[utoipa::path(
    get,
    tag = "Answers",
    path = "/api/answers",
    responses(
        (status = 200, body = Vec<AnswerResponse>),
        ApiError
    ),
    security(("jwt_token" = []))
)]
async fn list(
    _auth_user: AuthUser,
    State(ctx): State<AppContext>,
) -> axum::response::Result<Response> {
    let answers = answers::Entity::find()
        .all(&ctx.db)
        .await
        .map_err(ApiError::from)?
        .into_iter()
        .map(AnswerResponse::from)
        .collect::<Vec<_>>();

    Ok(Json(answers).into_response())
}

/// Get answer by id (requires auth, subscription if topic requires)
#[utoipa::path(
    get,
    tag = "Answers",
    path = "/api/answers/{id}",
    params(("id" = Uuid, Path, description = "Answer ID")),
    responses(
        (status = 200, body = AnswerResponse),
        ApiError
    ),
    security(("jwt_token" = []))
)]
async fn get(
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    State(ctx): State<AppContext>,
) -> axum::response::Result<Response> {
    let answer = answers::Entity::find_by_id(id)
        .one(&ctx.db)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;

    // Проверяем доступ к topic через question
    let topic_id = get_topic_id_by_question(&ctx.db, answer.question_id).await?;
    check_topic_access_by_id(&ctx.db, auth_user.user.id, topic_id).await?;

    Ok(Json(AnswerResponse::from(answer)).into_response())
}

/// Create answer (requires auth, subscription if topic requires)
#[utoipa::path(
    post,
    tag = "Answers",
    path = "/api/answers",
    request_body = CreateAnswerParams,
    responses(
        (status = 201, body = AnswerResponse),
        ApiError
    ),
    security(("jwt_token" = []))
)]
async fn create(
    auth_user: AuthUser,
    State(ctx): State<AppContext>,
    Json(params): Json<CreateAnswerParams>,
) -> axum::response::Result<Response> {
    params.validate().map_err(ApiError::from)?;

    // Проверяем доступ к topic через question
    let topic_id = get_topic_id_by_question(&ctx.db, params.question_id).await?;
    check_topic_access_by_id(&ctx.db, auth_user.user.id, topic_id).await?;

    let answer = answers::ActiveModel {
        question_id: Set(params.question_id),
        value: Set(params.value),
        is_correct: Set(params.is_correct),
        ..Default::default()
    };

    let answer = answer.insert(&ctx.db).await.map_err(ApiError::from)?;

    Ok((
        axum::http::StatusCode::CREATED,
        Json(AnswerResponse::from(answer)),
    )
        .into_response())
}

/// Update answer by id (requires auth, subscription if topic requires)
#[utoipa::path(
    patch,
    tag = "Answers",
    path = "/api/answers/{id}",
    params(("id" = Uuid, Path, description = "Answer ID")),
    request_body = UpdateAnswerParams,
    responses(
        (status = 200, body = AnswerResponse),
        ApiError
    ),
    security(("jwt_token" = []))
)]
async fn update(
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    State(ctx): State<AppContext>,
    Json(params): Json<UpdateAnswerParams>,
) -> axum::response::Result<Response> {
    params.validate().map_err(ApiError::from)?;

    let answer = answers::Entity::find_by_id(id)
        .one(&ctx.db)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;

    // Проверяем доступ к topic через текущий question
    let topic_id = get_topic_id_by_question(&ctx.db, answer.question_id).await?;
    check_topic_access_by_id(&ctx.db, auth_user.user.id, topic_id).await?;

    let mut to_update = answer.into_active_model();

    if let Some(question_id) = params.question_id {
        // Проверяем доступ к topic через новый question
        let new_topic_id = get_topic_id_by_question(&ctx.db, question_id).await?;
        check_topic_access_by_id(&ctx.db, auth_user.user.id, new_topic_id).await?;
        to_update.question_id = Set(question_id);
    }
    if let Some(value) = params.value {
        to_update.value = Set(value);
    }
    if let Some(is_correct) = params.is_correct {
        to_update.is_correct = Set(is_correct);
    }

    let answer = to_update.update(&ctx.db).await.map_err(ApiError::from)?;

    Ok(Json(AnswerResponse::from(answer)).into_response())
}

/// Delete answer by id (requires auth, subscription if topic requires)
#[utoipa::path(
    delete,
    tag = "Answers",
    path = "/api/answers/{id}",
    params(("id" = Uuid, Path, description = "Answer ID")),
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
    let answer = answers::Entity::find_by_id(id)
        .one(&ctx.db)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;

    // Проверяем доступ к topic через question
    let topic_id = get_topic_id_by_question(&ctx.db, answer.question_id).await?;
    check_topic_access_by_id(&ctx.db, auth_user.user.id, topic_id).await?;

    answers::Entity::delete_by_id(id)
        .exec(&ctx.db)
        .await
        .map_err(ApiError::from)?;

    Ok(().into_response())
}

/// Get answers by question id (requires auth, subscription if topic requires)
#[utoipa::path(
    get,
    tag = "Answers",
    path = "/api/questions/{question_id}/answers",
    params(("question_id" = Uuid, Path, description = "Question ID")),
    responses(
        (status = 200, body = Vec<AnswerResponse>),
        ApiError
    ),
    security(("jwt_token" = []))
)]
async fn get_by_question(
    auth_user: AuthUser,
    Path(question_id): Path<Uuid>,
    State(ctx): State<AppContext>,
) -> axum::response::Result<Response> {
    // Проверяем доступ к topic через question
    let topic_id = get_topic_id_by_question(&ctx.db, question_id).await?;
    check_topic_access_by_id(&ctx.db, auth_user.user.id, topic_id).await?;

    let answers = answers::Entity::find()
        .filter(answers::Column::QuestionId.eq(question_id))
        .all(&ctx.db)
        .await
        .map_err(ApiError::from)?
        .into_iter()
        .map(AnswerResponse::from)
        .collect::<Vec<_>>();

    Ok(Json(answers).into_response())
}

pub fn routes() -> OpenApiRouter<AppContext> {
    OpenApiRouter::new()
        .routes(routes!(list))
        .routes(routes!(get))
        .routes(routes!(create))
        .routes(routes!(update))
        .routes(routes!(delete))
        .routes(routes!(get_by_question))
}
