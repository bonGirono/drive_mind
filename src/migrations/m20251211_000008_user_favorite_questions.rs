use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::create()
            .table(UserFavoriteQuestions::Table)
            .if_not_exists()
            .col(uuid(UserFavoriteQuestions::UserId))
            .col(uuid(UserFavoriteQuestions::QuestionId))
            .primary_key(
                Index::create()
                    .col(UserFavoriteQuestions::UserId)
                    .col(UserFavoriteQuestions::QuestionId),
            )
            .foreign_key(
                ForeignKey::create()
                    .name("fk_user_favorite_questions_user")
                    .from(UserFavoriteQuestions::Table, UserFavoriteQuestions::UserId)
                    .to(Users::Table, Users::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade),
            )
            .foreign_key(
                ForeignKey::create()
                    .name("fk_user_favorite_questions_question")
                    .from(
                        UserFavoriteQuestions::Table,
                        UserFavoriteQuestions::QuestionId,
                    )
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
            .drop_table(Table::drop().table(UserFavoriteQuestions::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum UserFavoriteQuestions {
    Table,
    UserId,
    QuestionId,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}

#[derive(Iden)]
enum Questions {
    Table,
    Id,
}
