use data_model::Programme;

use super::ServiceError;
use crate::state::Deps;

/// Free-text programme search for a school.
///
/// # Errors
/// Returns [`ServiceError::Upstream`] if the school is unknown or all URLs fail.
// No result cache. Add a small Postgres-backed one if search traffic warrants it.
pub async fn search(
    deps: &Deps,
    school: &str,
    query: &str,
) -> Result<Vec<Programme>, ServiceError> {
    let school_config = deps.schools.get(school).ok_or(ServiceError::Upstream)?;
    for url in &school_config.urls {
        match deps.client.search_programmes(url, query).await {
            Ok(programmes) => return Ok(programmes),
            Err(error) => rocket::warn!("kronox programme search failed for {url}: {error}"),
        }
    }
    Err(ServiceError::Upstream)
}
