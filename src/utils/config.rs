use std::net::{IpAddr, SocketAddr};
use std::{fmt::Debug, net::Ipv4Addr};

use clap::Parser;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use super::db::{apply_migrations, create_sockets_connection};

#[derive(Parser, Debug, Clone)]
pub struct DBConfig {
    #[arg(long, env("DATABASE_URL"))]
    pub url: String,
    #[arg(long, env("DB_MAX_CONNECTIONS"), default_value_t = 10)]
    pub max_connections: u32,
    #[arg(long, env("DB_MIN_CONNECTIONS"), default_value_t = 1)]
    pub min_connections: u32,
    #[arg(long, env("DB_CONNECT_TIMEOUT"), default_value_t = 3)]
    pub connect_timeout: u64,
    #[arg(long, env("DB_ACQUIRE_TIMEOUT"), default_value_t = 3)]
    pub acquire_timeout: u64,
    #[arg(long, env("DB_IDLE_TIMEOUT"), default_value_t = 3)]
    pub idle_timeout: u64,
    #[arg(long, env("DB_MAX_LIFETIME"), default_value_t = 3)]
    pub max_lifetime: u64,
    #[arg(long, env("AUTO_MIGRATE"), default_value_t = false)]
    pub auto_migrate: bool,
}

impl DBConfig {
    pub fn load() -> Self {
        dotenv::dotenv().ok();
        Self::parse()
    }

    pub async fn connect(&self) -> DatabaseConnection {
        let db = create_sockets_connection(self.clone()).await;
        if self.auto_migrate {
            if let Err(err) = apply_migrations(&db).await {
                panic!("{:?}", err);
            }
        }
        db
    }
}

#[derive(Parser, Debug, Clone, Serialize, Deserialize)]
#[command(version, about, long_about = None)]
pub struct ServerConfig {
    #[arg(short, long, env("SERVER_IP"), default_value_t=Ipv4Addr::new(0, 0, 0, 0))]
    pub ip: Ipv4Addr,

    #[arg(short, long, env("SERVER_PORT"), default_value_t = 3030)]
    pub port: u16,

    // #[arg(long, env("SMTP_HOST"))]
    // pub smtp_host: String,

    // #[arg(long, env("SMTP_PORT"), default_value_t = 465)]
    // pub smtp_port: u16,

    // #[arg(long, env("SMTP_USER"))]
    // pub smtp_user: String,

    // #[arg(long, env("SMTP_PASSWORD"))]
    // pub smtp_password: String,
    #[arg(long, env("SECRET_KEY"))]
    pub secret_key: String,
}

impl ServerConfig {
    pub fn load() -> Self {
        Self::parse()
    }

    pub fn get_addr(&self) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(self.ip), self.port)
    }
}
