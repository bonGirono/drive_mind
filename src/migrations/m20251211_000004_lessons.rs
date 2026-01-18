use sea_orm_migration::{prelude::*, schema::*};

use crate::migrations::m20251211_000003_topics::Topics;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::create()
            .table(Lessons::Table)
            .if_not_exists()
            .col(
                pk_uuid(Lessons::Id)
                    .extra("DEFAULT gen_random_uuid()")
                    .primary_key(),
            )
            .col(uuid(Lessons::TopicId).unique_key())
            .foreign_key(
                ForeignKey::create()
                    .name("fk-topic-lesson")
                    .from(Lessons::Table, Lessons::TopicId)
                    .to(Topics::Table, Topics::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade),
            )
            .col(text(Lessons::Content))
            .to_owned();
        manager.create_table(table).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Lessons::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Lessons {
    Table,
    Id,
    TopicId,
    Content,
}
