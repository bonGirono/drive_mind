use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::create()
            .table(TestQuestions::Table)
            .if_not_exists()
            .col(uuid(TestQuestions::TestId))
            .col(uuid(TestQuestions::QuestionId))
            .col(small_integer(TestQuestions::QuestionOrder))
            .col(boolean_null(TestQuestions::IsCorrect))
            .col(timestamp_with_time_zone_null(TestQuestions::AnsweredAt))
            .primary_key(
                Index::create()
                    .col(TestQuestions::TestId)
                    .col(TestQuestions::QuestionId),
            )
            .foreign_key(
                ForeignKey::create()
                    .name("fk_test_questions_test")
                    .from(TestQuestions::Table, TestQuestions::TestId)
                    .to(Tests::Table, Tests::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade),
            )
            .foreign_key(
                ForeignKey::create()
                    .name("fk_test_questions_question")
                    .from(TestQuestions::Table, TestQuestions::QuestionId)
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
            .drop_table(Table::drop().table(TestQuestions::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum TestQuestions {
    Table,
    TestId,
    QuestionId,
    QuestionOrder,
    IsCorrect,
    AnsweredAt,
}

#[derive(Iden)]
enum Tests {
    Table,
    Id,
}

#[derive(Iden)]
enum Questions {
    Table,
    Id,
}
