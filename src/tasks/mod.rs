pub mod handle_subscriptions;

use apalis::prelude::{Data, Error};
use serde::{Deserialize, Serialize};

use crate::AppContext;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum TasksEnum {
    #[default]
    CheckSubscriptions,
}

pub async fn scheduled_task(
    ctx: TasksEnum,
    state: Data<AppContext>, // Extension(config): Extension<ServerConfig>,
) -> Result<(), Error> {
    match ctx {
        TasksEnum::CheckSubscriptions => {
            handle_subscriptions::disable_expired_subscriptions(&state.db).await
        }
    };
    Ok(())
}
