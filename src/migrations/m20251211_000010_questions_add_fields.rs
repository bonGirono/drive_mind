use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Добавляем поле content (optional)
        manager
            .alter_table(
                Table::alter()
                    .table(Questions::Table)
                    .add_column(string_null(Questions::Content))
                    .to_owned(),
            )
            .await?;

        // Добавляем поле explanation (required)
        manager
            .alter_table(
                Table::alter()
                    .table(Questions::Table)
                    .add_column(string(Questions::Explanation).default(""))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Questions::Table)
                    .drop_column(Questions::Content)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Questions::Table)
                    .drop_column(Questions::Explanation)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Questions {
    Table,
    Content,
    Explanation,
}
