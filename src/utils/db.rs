use std::time::Duration;

use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use sea_orm_migration::{MigratorTrait, SchemaManager};

use super::config::DBConfig;

pub async fn create_sockets_connection(config: DBConfig) -> DatabaseConnection {
    let mut opt = ConnectOptions::new(config.url.clone());
    opt.max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .connect_timeout(Duration::from_secs(config.connect_timeout))
        .acquire_timeout(Duration::from_secs(config.acquire_timeout))
        .idle_timeout(Duration::from_secs(config.idle_timeout))
        .max_lifetime(Duration::from_secs(config.max_lifetime));

    match Database::connect(opt).await {
        Ok(conn) => conn,
        Err(_) => panic!(
            "Couldn't connect to database\nCheck the database status: {}",
            config.url.clone()
        ),
    }
}

pub async fn apply_migrations(db: &DatabaseConnection) -> Result<(), DbErr> {
    let schema_manager = SchemaManager::new(&*db); // To investigate the schema
    crate::migrations::Migrator::up(&*db, None).await?;
    assert!(schema_manager.has_table("users").await?);

    Ok(())
}
