use sea_orm_migration::{prelude::*, schema::*};

use super::utils::table_auto_tz;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = table_auto_tz(Users::Table)
            .col(
                pk_uuid(Users::Id)
                    .extra("DEFAULT gen_random_uuid()")
                    .primary_key(),
            )
            .col(string(Users::Email))
            .col(string(Users::Password))
            .col(string_null(Users::PhoneNumber))
            .col(string_null(Users::Username))
            .to_owned();
        manager.create_table(table).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Users {
    Table,
    Id,
    Email,
    Password,
    PhoneNumber,
    Username,
}
