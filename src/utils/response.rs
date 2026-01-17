use std::sync::Arc;

use argon2::password_hash;
use axum::{
    Json,
    response::{IntoResponse, Response},
};
use hyper::StatusCode;
use lettre::address::AddressError;
use serde::Serialize;
use validator::{ValidationError, ValidationErrors};

#[derive(Debug, Serialize, utoipa::IntoResponses, thiserror::Error)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ApiError {
    // 400 - Bad Request
    #[response(status = 400, description = "BadRequest")]
    BadRequest,
    #[response(status = 400, description = "ValidationFailed")]
    ValidationFailed,
    #[response(status = 400, description = "InvalidInput")]
    InvalidInput,
    #[response(status = 400, description = "InvalidFormat")]
    InvalidFormat,
    #[response(status = 400, description = "MissingField")]
    MissingField,
    #[response(status = 400, description = "InvalidFieldValue")]
    InvalidFieldValue,
    #[response(status = 400, description = "InvalidSum")]
    InvalidSum,
    #[response(
        status = 400,
        description = "BadRequest | ValidationFailed | InvalidInput | InvalidFormat | MissingField | InvalidFieldValue | InvalidSum"
    )]
    Any400,

    // 401 - Unauthorized
    #[response(status = 401, description = "Unauthorized")]
    Unauthorized,
    #[response(status = 401, description = "InvalidCredentials")]
    InvalidCredentials,
    #[response(status = 401, description = "InvalidToken")]
    InvalidToken,
    #[response(status = 401, description = "TokenExpired")]
    TokenExpired,
    #[response(status = 401, description = "TokenMissing")]
    TokenMissing,
    #[response(
        status = 401,
        description = "Unauthorized | InvalidCredentials | InvalidToken | TokenExpired | TokenMissing"
    )]
    Any401,

    // 402 - PaymentRequired
    #[response(status = 402, description = "NoMoney")]
    NoMoney,
    #[response(status = 402, description = "PaymentRequired")]
    PaymentRequired,
    #[response(status = 402, description = "NoMoney | PaymentRequired")]
    Any402,

    // 403 - Forbidden
    #[response(status = 403, description = "Forbidden")]
    Forbidden,
    #[response(status = 403, description = "InsufficientPermissions")]
    InsufficientPermissions,
    #[response(status = 403, description = "AccessDenied")]
    AccessDenied,
    #[response(
        status = 403,
        description = "Forbidden | InsufficientPermissions | AccessDenied"
    )]
    Any403,

    // 404 - Not Found
    #[response(status = 404, description = "NotFound")]
    NotFound,
    #[response(status = 404, description = "UserNotFound")]
    UserNotFound,
    #[response(status = 404, description = "ResourceNotFound")]
    ResourceNotFound,
    #[response(status = 404, description = "EndpointNotFound")]
    EndpointNotFound,
    #[response(
        status = 404,
        description = "NotFound | UserNotFound | ResourceNotFound | EndpointNotFound"
    )]
    Any404,

    // 409 - Conflict
    #[response(status = 409, description = "Conflict")]
    Conflict,
    #[response(status = 409, description = "AlreadyExists")]
    AlreadyExists,
    #[response(status = 409, description = "DuplicateEntry")]
    DuplicateEntry,
    #[response(status = 409, description = "ResourceLocked")]
    ResourceLocked,
    #[response(
        status = 409,
        description = "Conflict | AlreadyExists | DuplicateEntry | ResourceLocked"
    )]
    Any409,

    // 422 - Unprocessable Entity
    #[response(status = 422, description = "UnprocessableEntity")]
    UnprocessableEntity,
    #[response(status = 422, description = "InvalidState")]
    InvalidState,
    #[response(status = 422, description = "UnprocessableEntity | InvalidState")]
    Any422,

    // 500 - Internal Server Error
    #[response(status = 500, description = "InternalServerError")]
    InternalServerError,
    #[response(status = 500, description = "DatabaseError")]
    DatabaseError,
    #[response(status = 500, description = "ServiceError")]
    ServiceError,
    #[response(status = 500, description = "ConfigurationError")]
    ConfigurationError,
    #[response(
        status = 500,
        description = "InternalServerError | DatabaseError | ServiceError | ConfigurationError"
    )]
    Any500,

    // 502 - Bad Gateway
    #[response(status = 502, description = "BadGateway")]
    BadGateway,
    #[response(status = 502, description = "ExternalServiceError")]
    ExternalServiceError,
    #[response(status = 502, description = "BadGateway | ExternalServiceError")]
    Any502,

    // 503 - Service Unavailable
    #[response(status = 503, description = "ServiceUnavailable")]
    ServiceUnavailable,
    #[response(status = 503, description = "MaintenanceMode")]
    MaintenanceMode,
    #[response(status = 503, description = "ServiceUnavailable | MaintenanceMode")]
    Any503,

    // 504 - Gateway Timeout
    #[response(status = 504, description = "GatewayTimeout")]
    GatewayTimeout,
}

impl ApiError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            ApiError::BadRequest
            | ApiError::ValidationFailed
            | ApiError::InvalidInput
            | ApiError::InvalidFormat
            | ApiError::MissingField
            | ApiError::InvalidFieldValue
            | ApiError::InvalidSum
            | ApiError::Any400 => StatusCode::BAD_REQUEST,

            ApiError::Unauthorized
            | ApiError::InvalidCredentials
            | ApiError::InvalidToken
            | ApiError::TokenExpired
            | ApiError::TokenMissing
            | ApiError::Any401 => StatusCode::UNAUTHORIZED,

            ApiError::NoMoney | ApiError::PaymentRequired | ApiError::Any402 => {
                StatusCode::PAYMENT_REQUIRED
            }

            ApiError::Forbidden
            | ApiError::InsufficientPermissions
            | ApiError::AccessDenied
            | ApiError::Any403 => StatusCode::FORBIDDEN,

            ApiError::NotFound
            | ApiError::UserNotFound
            | ApiError::ResourceNotFound
            | ApiError::EndpointNotFound
            | ApiError::Any404 => StatusCode::NOT_FOUND,

            ApiError::Conflict
            | ApiError::AlreadyExists
            | ApiError::DuplicateEntry
            | ApiError::ResourceLocked
            | ApiError::Any409 => StatusCode::CONFLICT,

            ApiError::UnprocessableEntity | ApiError::InvalidState | ApiError::Any422 => {
                StatusCode::UNPROCESSABLE_ENTITY
            }

            ApiError::InternalServerError
            | ApiError::DatabaseError
            | ApiError::ServiceError
            | ApiError::ConfigurationError
            | ApiError::Any500 => StatusCode::INTERNAL_SERVER_ERROR,

            ApiError::BadGateway | ApiError::ExternalServiceError | ApiError::Any502 => {
                StatusCode::BAD_GATEWAY
            }

            ApiError::ServiceUnavailable | ApiError::MaintenanceMode | ApiError::Any503 => {
                StatusCode::SERVICE_UNAVAILABLE
            }

            ApiError::GatewayTimeout => StatusCode::GATEWAY_TIMEOUT,
        }
    }

    pub fn message(&self) -> &str {
        match self {
            ApiError::BadRequest => "Bad request",
            ApiError::ValidationFailed => "Validation failed",
            ApiError::InvalidInput => "Invalid input provided",
            ApiError::InvalidFormat => "Invalid format",
            ApiError::MissingField => "Required field is missing",
            ApiError::InvalidFieldValue => "Invalid field value",
            ApiError::InvalidSum => "Invalid sum",
            ApiError::Any400 => "",

            ApiError::NoMoney => "No money, no honey",
            ApiError::PaymentRequired => "Payment required",
            ApiError::Any401 => "",

            ApiError::Unauthorized => "Unauthorized",
            ApiError::InvalidCredentials => "Invalid credentials",
            ApiError::InvalidToken => "Invalid authentication token",
            ApiError::TokenExpired => "Token has expired",
            ApiError::TokenMissing => "Authentication token is missing",
            ApiError::Any402 => "",

            ApiError::Forbidden => "Forbidden",
            ApiError::InsufficientPermissions => "Insufficient permissions",
            ApiError::AccessDenied => "Access denied",
            ApiError::Any403 => "",

            ApiError::NotFound => "Resource not found",
            ApiError::UserNotFound => "User not found",
            ApiError::ResourceNotFound => "Requested resource not found",
            ApiError::EndpointNotFound => "Endpoint not found",
            ApiError::Any404 => "",

            ApiError::Conflict => "Conflict",
            ApiError::AlreadyExists => "Resource already exists",
            ApiError::DuplicateEntry => "Duplicate entry",
            ApiError::ResourceLocked => "Resource is locked",
            ApiError::Any409 => "",

            ApiError::UnprocessableEntity => "Unprocessable entity",
            ApiError::InvalidState => "Invalid state",
            ApiError::Any422 => "",

            ApiError::InternalServerError => "Internal server error",
            ApiError::DatabaseError => "Database error occurred",
            ApiError::ServiceError => "Service error occurred",
            ApiError::ConfigurationError => "Configuration error",
            ApiError::Any500 => "",

            ApiError::BadGateway => "Bad gateway",
            ApiError::ExternalServiceError => "External service error",
            ApiError::Any502 => "",

            ApiError::ServiceUnavailable => "Service unavailable",
            ApiError::MaintenanceMode => "Service is in maintenance mode",
            ApiError::Any503 => "",

            ApiError::GatewayTimeout => "Gateway timeout",
        }
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: ApiError,
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let message = self.message().to_string();
        let status = self.status_code();

        let body = Json(ErrorResponse {
            error: self,
            message,
        });

        (status, body).into_response()
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.status_code(), self.message())
    }
}

impl From<uuid::Error> for ApiError {
    fn from(value: uuid::Error) -> Self {
        tracing::error!("invalid uuid - {:?}", value);
        Self::InvalidFieldValue
    }
}

impl From<sea_orm::DbErr> for ApiError {
    fn from(err: sea_orm::DbErr) -> Self {
        tracing::trace!("{err}");
        match err {
            sea_orm::DbErr::Conn(_) => ApiError::InternalServerError,

            sea_orm::DbErr::Exec(_) => ApiError::InternalServerError,

            sea_orm::DbErr::Query(_) => ApiError::InternalServerError,

            sea_orm::DbErr::Type(_) => ApiError::InternalServerError,

            sea_orm::DbErr::Json(_) => ApiError::InternalServerError,

            sea_orm::DbErr::RecordNotFound(_) => ApiError::NotFound,

            sea_orm::DbErr::RecordNotInserted => ApiError::InternalServerError,

            _ => ApiError::InternalServerError,
        }
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        tracing::error!("reqwest error: {}", err);
        Self::ExternalServiceError
    }
}

impl From<password_hash::Error> for ApiError {
    fn from(value: password_hash::Error) -> Self {
        tracing::error!("password_hash: {value}");
        Self::InternalServerError
    }
}

impl From<ApiError> for apalis::prelude::Error {
    fn from(value: ApiError) -> Self {
        Self::Failed(Arc::new(Box::new(value)))
    }
}

impl From<AddressError> for ApiError {
    fn from(value: AddressError) -> Self {
        tracing::error!("parse email error: {value}");
        Self::InvalidCredentials
    }
}

impl From<lettre::error::Error> for ApiError {
    fn from(value: lettre::error::Error) -> Self {
        tracing::error!("build email error: {value}");
        Self::InvalidCredentials
    }
}

impl From<lettre::transport::smtp::Error> for ApiError {
    fn from(value: lettre::transport::smtp::Error) -> Self {
        tracing::error!("send email error: {value}");
        Self::ExternalServiceError
    }
}

impl From<ValidationError> for ApiError {
    fn from(value: ValidationError) -> Self {
        tracing::error!("validation error: {value}");
        Self::ValidationFailed
    }
}

impl From<ValidationErrors> for ApiError {
    fn from(value: ValidationErrors) -> Self {
        tracing::error!("validation errors: {value}");
        Self::ValidationFailed
    }
}
