use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};

/// When a schedule was last refreshed from `KronoX`, if ever.
///
/// # Errors
/// Returns any error from the query.
pub async fn get_last_updated(
    pool: &PgPool,
    schedule_id: &str,
) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
    let row = sqlx::query("SELECT last_updated FROM schedule_meta WHERE schedule_id = $1")
        .bind(schedule_id)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|row| row.get("last_updated")))
}

/// Record a successful refresh timestamp for a schedule.
///
/// # Errors
/// Returns any error from the query.
pub async fn set_last_updated(
    pool: &PgPool,
    schedule_id: &str,
    school_code: &str,
    when: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO schedule_meta (schedule_id, school_code, last_updated) \
         VALUES ($1, $2, $3) \
         ON CONFLICT (schedule_id) DO UPDATE SET \
            school_code = EXCLUDED.school_code, last_updated = EXCLUDED.last_updated",
    )
    .bind(schedule_id)
    .bind(school_code)
    .bind(when)
    .execute(pool)
    .await?;
    Ok(())
}
