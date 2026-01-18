use crate::{
    AppContext,
    entities::user_favorite_questions,
    utils::{extractors::AuthUser, response::ApiError},
};
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait, ModelTrait};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

/// Add question to favorites
#[utoipa::path(
    post,
    tag = "User Favorite Questions",
    path = "/api/favorites/questions/{question_id}",
    params(("question_id" = Uuid, Path, description = "Question ID")),
    responses(
        (status = 201, description = "Added to favorites"),
        ApiError
    ),
    security(("jwt_token" = []))
)]
async fn add_favorite(
    auth_user: AuthUser,
    Path(question_id): Path<Uuid>,
    State(ctx): State<AppContext>,
) -> axum::response::Result<Response> {
    let favorite = user_favorite_questions::ActiveModel {
        user_id: Set(auth_user.user.id),
        question_id: Set(question_id),
    };

    favorite.insert(&ctx.db).await.map_err(ApiError::from)?;

    Ok(axum::http::StatusCode::CREATED.into_response())
}

/// Remove question from favorites
#[utoipa::path(
    delete,
    tag = "User Favorite Questions",
    path = "/api/favorites/questions/{question_id}",
    params(("question_id" = Uuid, Path, description = "Question ID")),
    responses(
        (status = 200, description = "Removed from favorites"),
        ApiError
    ),
    security(("jwt_token" = []))
)]
async fn remove_favorite(
    auth_user: AuthUser,
    Path(question_id): Path<Uuid>,
    State(ctx): State<AppContext>,
) -> axum::response::Result<Response> {
    let favorite = user_favorite_questions::Entity::find_by_id((auth_user.user.id, question_id))
        .one(&ctx.db)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;

    favorite.delete(&ctx.db).await.map_err(ApiError::from)?;

    Ok(().into_response())
}

pub fn routes() -> OpenApiRouter<AppContext> {
    OpenApiRouter::new()
        .routes(routes!(add_favorite))
        .routes(routes!(remove_favorite))
}
