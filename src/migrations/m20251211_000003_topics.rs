use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::create()
            .table(Topics::Table)
            .if_not_exists()
            .col(
                pk_uuid(Topics::Id)
                    .extra("DEFAULT gen_random_uuid()")
                    .primary_key(),
            )
            .col(string(Topics::Name))
            .col(string(Topics::Difficulty))
            .col(small_integer(Topics::Duration))
            .col(boolean(Topics::SubscriptionRequired).default(false))
            .to_owned();
        manager.create_table(table).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Topics::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Topics {
    Table,
    Id,
    Name,
    Difficulty,
    SubscriptionRequired,
    Duration,
}
