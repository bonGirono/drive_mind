use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::create()
            .table(Answers::Table)
            .if_not_exists()
            .col(
                pk_uuid(Answers::Id)
                    .extra("DEFAULT gen_random_uuid()")
                    .primary_key(),
            )
            .col(uuid(Answers::QuestionId))
            .col(string(Answers::Value))
            .col(boolean(Answers::IsCorrect))
            .col(string(Answers::Lang))
            .foreign_key(
                ForeignKey::create()
                    .name("fk_answers_question")
                    .from(Answers::Table, Answers::QuestionId)
                    .to(Questions::Table, Questions::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade),
            )
            .to_owned();
        manager.create_table(table).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Answers::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Answers {
    Table,
    Id,
    QuestionId,
    Value,
    IsCorrect,
    Lang,
}

#[derive(Iden)]
enum Questions {
    Table,
    Id,
}
