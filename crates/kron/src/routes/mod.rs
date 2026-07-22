pub mod programme;
pub mod schedule;

use rocket::serde::json::{Json, Value};

use crate::error::ApiError;
use crate::state::Deps;

pub(crate) type ApiResult = Result<Json<Value>, ApiError>;

pub(crate) fn require_school(deps: &Deps, school: Option<&str>) -> Result<String, ApiError> {
    match school {
        Some(code) if !code.is_empty() && deps.schools.get(code).is_some() => Ok(code.to_owned()),
        Some(code) if !code.is_empty() => Err(ApiError::UnknownSchool(deps.schools.allowed())),
        _ => Err(ApiError::BadRequest(
            "missing required query parameter: school".to_owned(),
        )),
    }
}

pub(crate) fn split_schedule_ids(value: Option<&str>) -> Result<Vec<String>, ApiError> {
    match value {
        Some(raw) if !raw.is_empty() => Ok(raw.split(',').map(|id| id.trim().to_owned()).collect()),
        _ => Err(ApiError::BadRequest(
            "missing required query parameter: schedule_ids".to_owned(),
        )),
    }
}
