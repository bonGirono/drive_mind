pub mod m20251211_000001_users;
pub mod m20251211_000002_user_subscriptions;
pub mod m20251211_000003_topics;
pub mod m20251211_000004_lessons;
pub mod m20251211_000005_images;
pub mod m20260118_000002_questions;
pub mod m20260118_000003_answers;
pub mod m20260118_000004_topics_subscription_required;
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
            Box::new(m20260118_000002_questions::Migration),
            Box::new(m20260118_000003_answers::Migration),
            Box::new(m20260118_000004_topics_subscription_required::Migration),
        ]
    }
}
