use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::create()
            .table(TestQuestionAnswers::Table)
            .if_not_exists()
            .col(uuid(TestQuestionAnswers::TestId))
            .col(uuid(TestQuestionAnswers::QuestionId))
            .col(uuid(TestQuestionAnswers::AnswerId))
            .primary_key(
                Index::create()
                    .col(TestQuestionAnswers::TestId)
                    .col(TestQuestionAnswers::QuestionId)
                    .col(TestQuestionAnswers::AnswerId),
            )
            .foreign_key(
                ForeignKey::create()
                    .name("fk_test_question_answers_test")
                    .from(TestQuestionAnswers::Table, TestQuestionAnswers::TestId)
                    .to(Tests::Table, Tests::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade),
            )
            .foreign_key(
                ForeignKey::create()
                    .name("fk_test_question_answers_question")
                    .from(TestQuestionAnswers::Table, TestQuestionAnswers::QuestionId)
                    .to(Questions::Table, Questions::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade),
            )
            .foreign_key(
                ForeignKey::create()
                    .name("fk_test_question_answers_answer")
                    .from(TestQuestionAnswers::Table, TestQuestionAnswers::AnswerId)
                    .to(Answers::Table, Answers::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade),
            )
            .to_owned();
        manager.create_table(table).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TestQuestionAnswers::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum TestQuestionAnswers {
    Table,
    TestId,
    QuestionId,
    AnswerId,
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

#[derive(Iden)]
enum Answers {
    Table,
    Id,
}
