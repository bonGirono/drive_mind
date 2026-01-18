use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::create()
            .table(Questions::Table)
            .if_not_exists()
            .col(
                pk_uuid(Questions::Id)
                    .extra("DEFAULT gen_random_uuid()")
                    .primary_key(),
            )
            .col(uuid(Questions::TopicId))
            .col(string(Questions::Name))
            .col(string(Questions::Lang))
            .foreign_key(
                ForeignKey::create()
                    .name("fk_questions_topic")
                    .from(Questions::Table, Questions::TopicId)
                    .to(Topics::Table, Topics::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade),
            )
            .to_owned();
        manager.create_table(table).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Questions::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Questions {
    Table,
    Id,
    TopicId,
    Name,
    Lang,
}

#[derive(Iden)]
enum Topics {
    Table,
    Id,
}
