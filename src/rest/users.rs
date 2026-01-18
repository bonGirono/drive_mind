use crate::{
    AppContext,
    entities::{user_subscriptions, users},
    models::users::{AuthParams, UpdateUserParams, UsersResponse},
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

/// Add subscription
#[utoipa::path(
    post,
    tag = "Users",
    path = "/api/users/_sub",
    request_body = AuthParams,
    responses(
        (status = 200),
        ApiError
    ),
    security()
)]
async fn add_sub(
    State(ctx): State<AppContext>,
    Json(params): Json<AuthParams>,
) -> axum::response::Result<Response> {
    let user = users::Entity::find_by_email(params.email)
        .one(&ctx.db)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::UserNotFound)?;

    user.validate_password(params.password)?;

    let sub = user_subscriptions::ActiveModel {
        user_id: Set(user.id),
        expire_at: Set((chrono::Utc::now() + chrono::Duration::hours(2)).into()),
        ..Default::default()
    };

    sub.insert(&ctx.db).await.map_err(ApiError::from)?;

    Ok(().into_response())
}

/// List users
#[utoipa::path(
    get,
    tag = "Users",
    path = "/api/users",
    responses(
        (status = 200, body = Vec<UsersResponse>),
        ApiError
    ),
    security()
)]
async fn list(State(ctx): State<AppContext>) -> axum::response::Result<Response> {
    Ok(Json(
        users::Entity::find()
            .all(&ctx.db)
            .await
            .map_err(ApiError::from)?
            .iter()
            .map(|v| UsersResponse::from(v.clone()))
            .collect::<Vec<UsersResponse>>(),
    )
    .into_response())
}

/// Get user by id
#[utoipa::path(
    get,
    tag = "Users",
    path = "/api/users/{id}",
    params(("id" = Uuid, Path, description = "Id")),
    responses(
        (status = 200, body = UsersResponse),
        ApiError
    ),
    security()
)]
async fn get(
    Path(id): Path<Uuid>,
    State(ctx): State<AppContext>,
) -> axum::response::Result<Response> {
    Ok(Json(UsersResponse::from(
        users::Entity::find_by_id(id)
            .one(&ctx.db)
            .await
            .map_err(ApiError::from)?
            .ok_or(ApiError::UserNotFound)?,
    ))
    .into_response())
}

/// Delete user by id
#[utoipa::path(
    delete,
    tag = "Users",
    path = "/api/users/{id}",
    params(("id" = Uuid, Path, description = "Id")),
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
    users::Entity::delete_by_id(id)
        .exec(&ctx.db)
        .await
        .map_err(ApiError::from)?;

    Ok(().into_response())
}

/// Update user by id
#[utoipa::path(
    patch,
    tag = "Users",
    path = "/api/users/{id}",
    params(("id" = Uuid, Path, description = "Id")),
    request_body = UpdateUserParams,
    responses(
        (status = 200, body = UsersResponse),
        ApiError
    ),
    security()
)]
async fn update(
    Path(id): Path<Uuid>,
    State(ctx): State<AppContext>,
    Json(params): Json<UpdateUserParams>,
) -> axum::response::Result<Response> {
    let user = users::Entity::find_by_id(id)
        .one(&ctx.db)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::UserNotFound)?;

    let mut to_update = user.into_active_model();

    if let Some(v) = params.username {
        to_update.username = Set(Some(v));
    }

    if let Some(v) = params.phone_number {
        to_update.phone_number = Set(Some(v));
    }

    if let Some(v) = params.email {
        to_update.email = Set(v);
    }

    let user = to_update.update(&ctx.db).await.map_err(ApiError::from)?;

    Ok(Json(UsersResponse::from(user)).into_response())
}

pub fn routes() -> OpenApiRouter<AppContext> {
    OpenApiRouter::new()
        .routes(routes!(list))
        .routes(routes!(get))
        .routes(routes!(delete))
        .routes(routes!(update))
        .routes(routes!(add_sub))
}
