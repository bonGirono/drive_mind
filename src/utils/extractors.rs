use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::{
    AppContext,
    entities::{topics, user_subscriptions, users},
    utils::{jwt::Claims, response::ApiError},
};

pub struct AuthUser {
    pub user: users::Model,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    AppContext: FromRef<S>,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let claims = Claims::from_request_parts(parts, state).await?;

        let ctx = AppContext::from_ref(state);

        let user = users::Entity::find_by_id(claims.id)
            .one(&ctx.db)
            .await
            .map_err(ApiError::from)?
            .ok_or(ApiError::UserNotFound)?;

        Ok(AuthUser { user })
    }
}

pub async fn check_topic_access(
    db: &DatabaseConnection,
    user_id: Uuid,
    topic: &topics::Model,
) -> Result<(), ApiError> {
    if !topic.subscription_required {
        return Ok(());
    }

    let now = chrono::Utc::now();
    let subscription = user_subscriptions::Entity::find()
        .filter(user_subscriptions::Column::UserId.eq(user_id))
        .filter(user_subscriptions::Column::IsActive.eq(true))
        .filter(user_subscriptions::Column::ExpireAt.gt(now))
        .one(db)
        .await
        .map_err(ApiError::from)?;

    if subscription.is_some() {
        Ok(())
    } else {
        Err(ApiError::PaymentRequired)
    }
}

pub async fn check_topic_access_by_id(
    db: &DatabaseConnection,
    user_id: Uuid,
    topic_id: Uuid,
) -> Result<topics::Model, ApiError> {
    let topic = topics::Entity::find_by_id(topic_id)
        .one(db)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;

    check_topic_access(db, user_id, &topic).await?;

    Ok(topic)
}
