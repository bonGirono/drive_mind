use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, prelude::Expr};

use crate::entities;

pub async fn disable_expired_subscriptions(db: &DatabaseConnection) {
    match entities::user_subscriptions::Entity::update_many()
        .col_expr(
            entities::user_subscriptions::Column::IsActive,
            Expr::value(false),
        )
        .filter(entities::user_subscriptions::Column::ExpireAt.lt(chrono::Utc::now()))
        .filter(entities::user_subscriptions::Column::IsActive.eq(true))
        .filter(entities::user_subscriptions::Column::IsDeleted.eq(false))
        .exec(db)
        .await
    {
        Ok(v) => tracing::info!(
            "disable_expired_subscriptions rows affected: {}",
            v.rows_affected
        ),
        Err(err) => tracing::error!("disable_expired_subscriptions: {err}"),
    };
}
