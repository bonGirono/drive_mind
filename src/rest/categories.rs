use crate::{
    AppContext,
    entities::categories,
    models::categories::{CategoryResponse, CreateCategoryParams, UpdateCategoryParams},
    utils::{extractors::AuthUser, response::ApiError},
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

/// List all categories
#[utoipa::path(
    get,
    tag = "Categories",
    path = "/api/categories",
    responses(
        (status = 200, body = Vec<CategoryResponse>),
        ApiError
    ),
    security(("jwt_token" = []))
)]
async fn list(
    _auth_user: AuthUser,
    State(ctx): State<AppContext>,
) -> axum::response::Result<Response> {
    let categories = categories::Entity::find()
        .all(&ctx.db)
        .await
        .map_err(ApiError::from)?
        .into_iter()
        .map(CategoryResponse::from)
        .collect::<Vec<_>>();

    Ok(Json(categories).into_response())
}

/// Get category by id
#[utoipa::path(
    get,
    tag = "Categories",
    path = "/api/categories/{id}",
    params(("id" = Uuid, Path, description = "Category ID")),
    responses(
        (status = 200, body = CategoryResponse),
        ApiError
    ),
    security(("jwt_token" = []))
)]
async fn get(
    _auth_user: AuthUser,
    Path(id): Path<Uuid>,
    State(ctx): State<AppContext>,
) -> axum::response::Result<Response> {
    let category = categories::Entity::find_by_id(id)
        .one(&ctx.db)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;

    Ok(Json(CategoryResponse::from(category)).into_response())
}

/// Create category
#[utoipa::path(
    post,
    tag = "Categories",
    path = "/api/categories",
    request_body = CreateCategoryParams,
    responses(
        (status = 201, body = CategoryResponse),
        ApiError
    ),
    security(("jwt_token" = []))
)]
async fn create(
    _auth_user: AuthUser,
    State(ctx): State<AppContext>,
    Json(params): Json<CreateCategoryParams>,
) -> axum::response::Result<Response> {
    params.validate().map_err(ApiError::from)?;

    let category = categories::ActiveModel {
        name: Set(params.name),
        ..Default::default()
    };

    let category = category.insert(&ctx.db).await.map_err(ApiError::from)?;

    Ok((
        axum::http::StatusCode::CREATED,
        Json(CategoryResponse::from(category)),
    )
        .into_response())
}

/// Update category by id
#[utoipa::path(
    patch,
    tag = "Categories",
    path = "/api/categories/{id}",
    params(("id" = Uuid, Path, description = "Category ID")),
    request_body = UpdateCategoryParams,
    responses(
        (status = 200, body = CategoryResponse),
        ApiError
    ),
    security(("jwt_token" = []))
)]
async fn update(
    _auth_user: AuthUser,
    Path(id): Path<Uuid>,
    State(ctx): State<AppContext>,
    Json(params): Json<UpdateCategoryParams>,
) -> axum::response::Result<Response> {
    params.validate().map_err(ApiError::from)?;

    let category = categories::Entity::find_by_id(id)
        .one(&ctx.db)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;

    let mut to_update = category.into_active_model();

    if let Some(name) = params.name {
        to_update.name = Set(name);
    }

    let category = to_update.update(&ctx.db).await.map_err(ApiError::from)?;

    Ok(Json(CategoryResponse::from(category)).into_response())
}

/// Delete category by id
#[utoipa::path(
    delete,
    tag = "Categories",
    path = "/api/categories/{id}",
    params(("id" = Uuid, Path, description = "Category ID")),
    responses(
        (status = 200),
        ApiError
    ),
    security(("jwt_token" = []))
)]
async fn delete(
    _auth_user: AuthUser,
    Path(id): Path<Uuid>,
    State(ctx): State<AppContext>,
) -> axum::response::Result<Response> {
    categories::Entity::delete_by_id(id)
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
