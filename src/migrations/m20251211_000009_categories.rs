use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Таблица категорий
        manager
            .create_table(
                Table::create()
                    .table(Categories::Table)
                    .if_not_exists()
                    .col(
                        pk_uuid(Categories::Id)
                            .extra("DEFAULT gen_random_uuid()")
                            .primary_key(),
                    )
                    .col(string(Categories::Name))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Categories::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(Iden)]
pub enum Categories {
    Table,
    Id,
    Name,
}
