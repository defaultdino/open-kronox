//! The schedule data path.
//!
//! Without a database, every request scrapes `KronoX` live. With a db, reads are
//! served from the Postgres cache

use chrono::{Duration, Utc};
use data_model::Event;
use kronox::{Client, School};
use sqlx::PgPool;

use super::ServiceError;
use super::filter::{self, Filter};
use crate::state::{Deps, RefreshLocks};

const STALE_AFTER: Duration = Duration::hours(6);

/// Read events (cache-first when a database is configured) then filter them.
///
/// # Errors
/// Returns [`ServiceError`] if the scrape or a database access fails.
pub async fn query_events(
    deps: &Deps,
    schedule_ids: &[String],
    school: &str,
    filter: &Filter,
) -> Result<Vec<Event>, ServiceError> {
    let events = load(deps, schedule_ids, school).await?;
    Ok(filter::apply(events, filter))
}

async fn load(
    deps: &Deps,
    schedule_ids: &[String],
    school: &str,
) -> Result<Vec<Event>, ServiceError> {
    let Some(pool) = &deps.pool else {
        return scrape(deps, schedule_ids, school).await;
    };

    let cached = storage_pg::get_events_by_schedule_ids(pool, schedule_ids)
        .await
        .map_err(|_| ServiceError::Db)?;

    if cached.is_empty() {
        return fetch_and_store(deps, pool, schedule_ids, school).await;
    }

    spawn_refresh_if_stale(deps, schedule_ids, school);
    Ok(cached)
}

async fn scrape(
    deps: &Deps,
    schedule_ids: &[String],
    school: &str,
) -> Result<Vec<Event>, ServiceError> {
    let school_config = deps.schools.get(school).ok_or(ServiceError::Upstream)?;
    fetch_with_failover(&deps.client, school_config, school, schedule_ids).await
}

async fn fetch_and_store(
    deps: &Deps,
    pool: &PgPool,
    schedule_ids: &[String],
    school: &str,
) -> Result<Vec<Event>, ServiceError> {
    let events = scrape(deps, schedule_ids, school).await?;

    storage_pg::upsert_events(pool, &events)
        .await
        .map_err(|_| ServiceError::Db)?;

    let now = Utc::now();
    for id in schedule_ids {
        let _ = storage_pg::set_last_updated(pool, id, school, now).await;
    }
    Ok(events)
}

async fn fetch_with_failover(
    client: &Client,
    school: &School,
    school_code: &str,
    schedule_ids: &[String],
) -> Result<Vec<Event>, ServiceError> {
    for url in &school.urls {
        match client
            .fetch_events(url, school_code, schedule_ids, None)
            .await
        {
            Ok(events) => {
                let events_len = events.len();
                log::info!("fetched {events_len} events from {url}");
                return Ok(events);
            }
            Err(error) => log::error!("kronox fetch failed for {url}: {error}"),
        }
    }
    Err(ServiceError::Upstream)
}

fn spawn_refresh_if_stale(deps: &Deps, schedule_ids: &[String], school: &str) {
    let deps = deps.clone();
    let schedule_ids = schedule_ids.to_vec();
    let school = school.to_owned();
    rocket::tokio::spawn(async move {
        let Some(pool) = deps.pool.clone() else {
            return;
        };
        if !is_stale(&pool, &schedule_ids).await {
            return;
        }
        let key = schedule_ids.join(",");
        if !acquire_lock(&deps.locks, &key) {
            return;
        }
        let _ = fetch_and_store(&deps, &pool, &schedule_ids, &school).await;
        release_lock(&deps.locks, &key);
    });
}

async fn is_stale(pool: &PgPool, schedule_ids: &[String]) -> bool {
    let cutoff = Utc::now() - STALE_AFTER;
    for id in schedule_ids {
        match storage_pg::get_last_updated(pool, id).await {
            Ok(Some(last)) if last > cutoff => {}
            Ok(_) => return true,
            Err(_) => return false,
        }
    }
    false
}

fn acquire_lock(locks: &RefreshLocks, key: &str) -> bool {
    locks
        .lock()
        .expect("refresh lock poisoned")
        .insert(key.to_owned())
}

fn release_lock(locks: &RefreshLocks, key: &str) {
    locks.lock().expect("refresh lock poisoned").remove(key);
}

pub async fn refresh_all_known(deps: &Deps) {
    let Some(pool) = &deps.pool else {
        return;
    };
    let refs = match storage_pg::distinct_schedule_refs(pool).await {
        Ok(refs) => refs,
        Err(error) => {
            rocket::error!("background refresh: listing schedules failed: {error}");
            return;
        }
    };
    for schedule_ref in refs {
        let ids = [schedule_ref.schedule_id];
        let _ = fetch_and_store(deps, pool, &ids, &schedule_ref.school_code).await;
    }
}
