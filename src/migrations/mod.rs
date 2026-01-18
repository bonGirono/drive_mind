pub mod m20251211_000001_users;
pub mod m20251211_000002_user_subscriptions;
use sea_orm_migration::prelude::*;
mod utils;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20251211_000001_users::Migration),
            Box::new(m20251211_000002_user_subscriptions::Migration),
        ]
    }
}
