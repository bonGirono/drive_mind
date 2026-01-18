use crate::{
    AppContext,
    entities::{categories, question_categories, questions},
    models::{categories::CategoryResponse, question_categories::QuestionCategoryResponse},
    utils::{extractors::AuthUser, response::ApiError},
};
use axum::{
    Json,
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, ModelTrait, QueryFilter,
};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

/// Get categories for a question
#[utoipa::path(
    get,
    tag = "Question Categories",
    path = "/api/questions/{question_id}/categories",
    params(("question_id" = Uuid, Path, description = "Question ID")),
    responses(
        (status = 200, body = Vec<CategoryResponse>),
        ApiError
    ),
    security(("jwt_token" = []))
)]
async fn get_question_categories(
    _auth_user: AuthUser,
    Path(question_id): Path<Uuid>,
    State(ctx): State<AppContext>,
) -> axum::response::Result<Response> {
    // Проверяем что вопрос существует
    questions::Entity::find_by_id(question_id)
        .one(&ctx.db)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;

    let category_ids: Vec<Uuid> = question_categories::Entity::find()
        .filter(question_categories::Column::QuestionId.eq(question_id))
        .all(&ctx.db)
        .await
        .map_err(ApiError::from)?
        .into_iter()
        .map(|qc| qc.category_id)
        .collect();

    let categories_list = categories::Entity::find()
        .filter(categories::Column::Id.is_in(category_ids))
        .all(&ctx.db)
        .await
        .map_err(ApiError::from)?
        .into_iter()
        .map(CategoryResponse::from)
        .collect::<Vec<_>>();

    Ok(Json(categories_list).into_response())
}

/// Add category to question
#[utoipa::path(
    post,
    tag = "Question Categories",
    path = "/api/questions/{question_id}/categories/{category_id}",
    params(
        ("question_id" = Uuid, Path, description = "Question ID"),
        ("category_id" = Uuid, Path, description = "Category ID")
    ),
    responses(
        (status = 201, body = QuestionCategoryResponse),
        ApiError
    ),
    security(("jwt_token" = []))
)]
async fn add_category_to_question(
    _auth_user: AuthUser,
    Path((question_id, category_id)): Path<(Uuid, Uuid)>,
    State(ctx): State<AppContext>,
) -> axum::response::Result<Response> {
    // Проверяем что вопрос существует
    questions::Entity::find_by_id(question_id)
        .one(&ctx.db)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;

    // Проверяем что категория существует
    categories::Entity::find_by_id(category_id)
        .one(&ctx.db)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;

    let link = question_categories::ActiveModel {
        question_id: Set(question_id),
        category_id: Set(category_id),
    };

    link.insert(&ctx.db).await.map_err(ApiError::from)?;

    Ok((
        axum::http::StatusCode::CREATED,
        Json(QuestionCategoryResponse {
            question_id,
            category_id,
        }),
    )
        .into_response())
}

/// Remove category from question
#[utoipa::path(
    delete,
    tag = "Question Categories",
    path = "/api/questions/{question_id}/categories/{category_id}",
    params(
        ("question_id" = Uuid, Path, description = "Question ID"),
        ("category_id" = Uuid, Path, description = "Category ID")
    ),
    responses(
        (status = 200, description = "Category removed from question"),
        ApiError
    ),
    security(("jwt_token" = []))
)]
async fn remove_category_from_question(
    _auth_user: AuthUser,
    Path((question_id, category_id)): Path<(Uuid, Uuid)>,
    State(ctx): State<AppContext>,
) -> axum::response::Result<Response> {
    let link = question_categories::Entity::find_by_id((question_id, category_id))
        .one(&ctx.db)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;

    link.delete(&ctx.db).await.map_err(ApiError::from)?;

    Ok(().into_response())
}

pub fn routes() -> OpenApiRouter<AppContext> {
    OpenApiRouter::new()
        .routes(routes!(get_question_categories))
        .routes(routes!(add_category_to_question))
        .routes(routes!(remove_category_from_question))
}
