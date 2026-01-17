use apalis::prelude::{Data, Error};
use serde::{Deserialize, Serialize};

use crate::AppContext;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TasksEnum {
    Healthcheck,
}

pub async fn scheduled_task(
    ctx: TasksEnum,
    state: Data<AppContext>, // Extension(config): Extension<ServerConfig>,
) -> Result<(), Error> {
    match ctx {
        TasksEnum::Healthcheck => tracing::info!("ok!"),
    };
    _ = state.tasks_redis_storage;
    Ok(())
}
