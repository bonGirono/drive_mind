use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelBehavior, ActiveModelTrait, ColumnTrait, ConnectionTrait, DbErr, EntityTrait,
    QueryFilter,
};
use sea_orm_migration::async_trait;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::entities::users;
use crate::utils::password::hash_password;
use crate::utils::response::ApiError;

#[derive(Clone, Debug, Validate, Serialize, Deserialize, ToSchema)]
pub struct AuthParams {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 3))]
    pub password: String,
}

#[derive(Clone, Debug, Validate, Serialize, Deserialize, ToSchema)]
pub struct UpdateUserParams {
    pub username: Option<String>,
    pub phone_number: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct UsersResponse {
    pub id: Uuid,
    pub email: String,
    pub phone_number: Option<String>,
    pub username: Option<String>,
}

impl From<crate::entities::users::Model> for UsersResponse {
    fn from(value: crate::entities::users::Model) -> Self {
        Self {
            id: value.id,
            email: value.email,
            phone_number: value.phone_number,
            username: value.username,
        }
    }
}

#[async_trait::async_trait]
impl ActiveModelBehavior for crate::entities::users::ActiveModel {
    async fn before_save<C>(self, _db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        // self.validate()?;
        if !insert && self.updated_at.is_unchanged() {
            let mut this = self;
            this.updated_at = sea_orm::ActiveValue::Set(chrono::Utc::now().into());
            Ok(this)
        } else {
            Ok(self)
        }
    }
}

impl crate::entities::users::Model {
    pub async fn create_with_password(
        db: &impl ConnectionTrait,
        params: &AuthParams,
    ) -> Result<Self, ApiError> {
        params.validate()?;

        if users::Entity::find_by_email(params.email.clone())
            .one(db)
            .await?
            .is_some()
        {
            return Err(ApiError::DuplicateEntry);
        }

        let password_hash = hash_password(&params.password)?;
        let user = users::ActiveModel {
            email: Set(params.email.to_string()),
            password: Set(password_hash),
            ..Default::default()
        }
        .insert(db)
        .await?;

        Ok(user)
    }

    pub fn validate_password(&self, password: String) -> Result<(), ApiError> {
        if crate::utils::password::verify_password(password, self.password.clone())? {
            Ok(())
        } else {
            Err(ApiError::Unauthorized)
        }
    }
}

impl users::Entity {
    pub fn find_by_email(email: String) -> sea_orm::Select<Self> {
        Self::find().filter(users::Column::Email.eq(email))
    }
}
