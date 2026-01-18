use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::create()
            .table(QuestionCategories::Table)
            .if_not_exists()
            .col(uuid(QuestionCategories::QuestionId))
            .col(uuid(QuestionCategories::CategoryId))
            .primary_key(
                Index::create()
                    .col(QuestionCategories::QuestionId)
                    .col(QuestionCategories::CategoryId),
            )
            .foreign_key(
                ForeignKey::create()
                    .name("fk_question_categories_question")
                    .from(QuestionCategories::Table, QuestionCategories::QuestionId)
                    .to(Questions::Table, Questions::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade),
            )
            .foreign_key(
                ForeignKey::create()
                    .name("fk_question_categories_category")
                    .from(QuestionCategories::Table, QuestionCategories::CategoryId)
                    .to(Categories::Table, Categories::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade),
            )
            .to_owned();
        manager.create_table(table).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(QuestionCategories::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum QuestionCategories {
    Table,
    QuestionId,
    CategoryId,
}

#[derive(Iden)]
enum Questions {
    Table,
    Id,
}

#[derive(Iden)]
enum Categories {
    Table,
    Id,
}
