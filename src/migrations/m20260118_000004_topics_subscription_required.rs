use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Topics::Table)
                    .add_column(boolean(Topics::SubscriptionRequired).default(false))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Topics::Table)
                    .drop_column(Topics::SubscriptionRequired)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum Topics {
    Table,
    SubscriptionRequired,
}
