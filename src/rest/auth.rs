use crate::{
    AppContext,
    entities::users,
    models::users::{AuthParams, UsersResponse},
    utils::{
        jwt::{AuthBody, Claims, KEYS},
        response::ApiError,
    },
};
use axum::{
    Json,
    extract::State,
    response::{IntoResponse, Response},
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, TransactionTrait};
use utoipa_axum::{router::OpenApiRouter, routes};

/// Get current user full data
#[utoipa::path(
    get,
    tag = "Auth",
    path = "/api/auth/current",
    responses(
        (status = 200, body = UsersResponse),
        ApiError
    ),
    security(
        ("jwt_token" = [])
    )
)]
async fn current(auth: Claims, State(ctx): State<AppContext>) -> axum::response::Result<Response> {
    match users::Entity::find()
        .filter(users::Column::Id.eq(auth.id))
        .one(&ctx.db)
        .await
    {
        Ok(Some(v)) => Ok(axum::Json(UsersResponse::from(v)).into_response()),
        Ok(None) => Err(ApiError::UserNotFound.into()),
        Err(err) => Err(ApiError::from(err).into()),
    }
}

/// Register account
#[utoipa::path(
    post,
    tag = "Auth",
    path = "/api/auth/register",
    request_body = AuthParams,
    responses(
        (status = 200),
        ApiError
    ),
    security()
)]
async fn register(
    State(ctx): State<AppContext>,
    // Extension(config): Extension<ServerConfig>,
    Json(params): Json<AuthParams>,
) -> axum::response::Result<Response> {
    let txn = ctx.db.begin().await.map_err(ApiError::from)?;
    let _user = users::Model::create_with_password(&txn, &params).await?;
    txn.commit().await.map_err(ApiError::from)?;

    // let mut background_tasks = ctx.tasks_redis_storage.clone();
    // let job = TasksEnum::SendWelcomeEmail(SendWelcomeData {
    //     config: config.clone(),
    //     user,
    // });
    // _ = background_tasks.push(job).await;

    Ok(().into_response())
}

/// Login account
#[utoipa::path(
    post,
    tag = "Auth",
    path = "/api/auth/login",
    request_body = AuthParams,
    responses(
        (status = 200, body = AuthBody),
        ApiError
    ),
    security()
)]
async fn login(
    State(ctx): State<AppContext>,
    Json(params): Json<AuthParams>,
) -> axum::response::Result<Response> {
    let user = users::Entity::find_by_email(params.email)
        .one(&ctx.db)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::UserNotFound)?;

    user.validate_password(params.password)?;

    let claims = Claims {
        id: user.id,
        is_admin: true,
        computer_id: None,
        room_id: None,
        exp: (chrono::Utc::now() + chrono::Duration::days(1)).timestamp() as usize, // May 2033
    };

    let token = jsonwebtoken::encode(&jsonwebtoken::Header::default(), &claims, &KEYS.encoding)
        .map_err(|_| ApiError::Unauthorized)?;

    let token = AuthBody::new(token);
    Ok(Json(token).into_response())
}

pub fn routes() -> OpenApiRouter<AppContext> {
    OpenApiRouter::new()
        .routes(routes!(current))
        .routes(routes!(register))
        .routes(routes!(login))
}
