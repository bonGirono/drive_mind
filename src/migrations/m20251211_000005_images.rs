use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::create()
            .table(Images::Table)
            .if_not_exists()
            .col(
                pk_uuid(Images::Id)
                    .extra("DEFAULT gen_random_uuid()")
                    .primary_key(),
            )
            .col(string(Images::OriginalName))
            .col(string(Images::StoredName))
            .col(string(Images::MimeType))
            .col(big_integer(Images::Size))
            .to_owned();
        manager.create_table(table).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Images::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Images {
    Table,
    Id,
    OriginalName,
    StoredName,
    MimeType,
    Size,
}
