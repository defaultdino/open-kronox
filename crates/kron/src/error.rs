use rocket::response::Responder;
use rocket::serde::json::{Json, Value, json};
use rocket::{Request, http::Status, response};

use crate::service::ServiceError;

pub enum ApiError {
    BadRequest(String),
    UnknownSchool(Vec<String>),
    Upstream,
    Internal,
}

impl ApiError {
    fn into_parts(self) -> (Status, Value) {
        match self {
            ApiError::BadRequest(message) => (Status::BadRequest, json!({ "error": message })),
            ApiError::UnknownSchool(allowed) => (
                Status::BadRequest,
                json!({ "error": "unknown school", "allowed_schools": allowed }),
            ),
            ApiError::Upstream => (
                Status::ServiceUnavailable,
                json!({ "error": "upstream unavailable", "retry_after": "300" }),
            ),
            ApiError::Internal => (
                Status::InternalServerError,
                json!({ "error": "internal error" }),
            ),
        }
    }
}

impl<'r> Responder<'r, 'static> for ApiError {
    fn respond_to(self, request: &'r Request<'_>) -> response::Result<'static> {
        let (status, body) = self.into_parts();
        (status, Json(body)).respond_to(request)
    }
}

impl From<ServiceError> for ApiError {
    fn from(error: ServiceError) -> Self {
        match error {
            ServiceError::Upstream => ApiError::Upstream,
            ServiceError::Db => ApiError::Internal,
        }
    }
}
