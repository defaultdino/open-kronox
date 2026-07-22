use rocket::State;
use rocket::serde::json::{Json, json};

use crate::error::ApiError;
use crate::routes::{ApiResult, require_school};
use crate::service;
use crate::state::Deps;

#[get("/programme/search?<search_query>&<school>")]
pub async fn search(
    deps: &State<Deps>,
    search_query: Option<&str>,
    school: Option<&str>,
) -> ApiResult {
    let school = require_school(deps, school)?;
    let query = match search_query {
        Some(text) if !text.is_empty() => text,
        _ => {
            return Err(ApiError::BadRequest(
                "missing required query parameter: search_query".to_owned(),
            ));
        }
    };

    let programmes = service::search_programmes(deps, &school, query).await?;
    let count = programmes.len();
    Ok(Json(json!({ "programmes": programmes, "count": count })))
}
