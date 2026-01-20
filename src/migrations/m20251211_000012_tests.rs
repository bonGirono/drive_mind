use sea_orm_migration::{prelude::*, schema::*};

use super::utils::table_auto_tz;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = table_auto_tz(Tests::Table)
            .col(
                pk_uuid(Tests::Id)
                    .extra("DEFAULT gen_random_uuid()")
                    .primary_key(),
            )
            .col(uuid(Tests::UserId))
            .col(string(Tests::FilterType))
            .col(uuid_null(Tests::FilterId))
            .col(string(Tests::Lang))
            .col(string(Tests::FilterHash))
            .col(small_integer(Tests::TotalQuestions))
            .col(small_integer(Tests::CorrectCount).default(0))
            .col(string(Tests::Status).default("active"))
            .col(small_integer_null(Tests::ScorePercent))
            .col(timestamp_with_time_zone_null(Tests::CompletedAt))
            .foreign_key(
                ForeignKey::create()
                    .name("fk_tests_user")
                    .from(Tests::Table, Tests::UserId)
                    .to(Users::Table, Users::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade),
            )
            .to_owned();
        manager.create_table(table).await?;

        // Partial unique index: only one active test per filter_hash per user
        // manager
        //     .create_index(
        //         Index::create()
        //             .name("idx_tests_user_filter_hash_active")
        //             .table(Tests::Table)
        //             .col(Tests::UserId)
        //             .col(Tests::FilterHash)
        //             .unique()
        //             .if_not_exists()
        //             .to_owned(),
        //     )
        //     .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // manager
        //     .drop_index(
        //         Index::drop()
        //             .name("idx_tests_user_filter_hash_active")
        //             .table(Tests::Table)
        //             .to_owned(),
        //     )
        //     .await?;
        manager
            .drop_table(Table::drop().table(Tests::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Tests {
    Table,
    Id,
    UserId,
    FilterType,
    FilterId,
    Lang,
    FilterHash,
    TotalQuestions,
    CorrectCount,
    Status,
    ScorePercent,
    CompletedAt,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}
