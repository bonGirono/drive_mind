pub mod m20251211_000001_users;
pub mod m20251211_000002_user_subscriptions;
pub mod m20251211_000003_topics;
pub mod m20251211_000004_lessons;
pub mod m20251211_000005_images;
pub mod m20251211_000006_questions;
pub mod m20251211_000007_answers;
pub mod m20251211_000008_user_favorite_questions;
pub mod m20251211_000009_categories;
use sea_orm_migration::prelude::*;
mod utils;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20251211_000001_users::Migration),
            Box::new(m20251211_000002_user_subscriptions::Migration),
            Box::new(m20251211_000003_topics::Migration),
            Box::new(m20251211_000004_lessons::Migration),
            Box::new(m20251211_000005_images::Migration),
            Box::new(m20251211_000006_questions::Migration),
            Box::new(m20251211_000007_answers::Migration),
            Box::new(m20251211_000008_user_favorite_questions::Migration),
            Box::new(m20251211_000009_categories::Migration),
        ]
    }
}
