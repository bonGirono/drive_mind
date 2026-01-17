pub mod m20251211_000001_users;
use sea_orm_migration::prelude::*;
mod utils;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20251211_000001_users::Migration)]
    }
}
