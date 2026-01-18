use sea_orm_migration::{prelude::*, schema::*};

use crate::migrations::m20251211_000001_users::Users;

use super::utils::table_auto_tz;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = table_auto_tz(UserSubscriptions::Table)
            .col(
                pk_uuid(UserSubscriptions::Id)
                    .extra("DEFAULT gen_random_uuid()")
                    .primary_key(),
            )
            .col(uuid(UserSubscriptions::UserId))
            .foreign_key(
                ForeignKey::create()
                    .name("fk-user-subscriptions")
                    .from(UserSubscriptions::Table, UserSubscriptions::UserId)
                    .to(Users::Table, Users::Id),
            )
            .col(timestamp_with_time_zone(UserSubscriptions::ExpireAt))
            .to_owned();
        manager.create_table(table).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserSubscriptions::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum UserSubscriptions {
    Table,
    Id,
    UserId,
    ExpireAt,
}
