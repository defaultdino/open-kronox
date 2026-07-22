use data_model::{Event, Location, Teacher};
use sqlx::types::Json;
use sqlx::{PgPool, Row};

#[derive(Debug, Clone)]
pub struct ScheduleRef {
    pub schedule_id: String,
    pub school_code: String,
}

/// Upsert events by `event_id`, one statement per row inside a transaction.
///
/// # Errors
/// Returns any error from the database or transaction.
// Batch via UNNEST if a schedule's event count ever makes per-row a bottleneck.
pub async fn upsert_events(pool: &PgPool, events: &[Event]) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    for event in events {
        sqlx::query(
            "INSERT INTO events (event_id, schedule_id, title, course_id, course_name, \
                teachers, locations, start_at, end_at, last_modified, is_special, \
                school_code, color) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13) \
             ON CONFLICT (event_id) DO UPDATE SET \
                schedule_id = EXCLUDED.schedule_id, title = EXCLUDED.title, \
                course_id = EXCLUDED.course_id, course_name = EXCLUDED.course_name, \
                teachers = EXCLUDED.teachers, locations = EXCLUDED.locations, \
                start_at = EXCLUDED.start_at, end_at = EXCLUDED.end_at, \
                last_modified = EXCLUDED.last_modified, is_special = EXCLUDED.is_special, \
                school_code = EXCLUDED.school_code, color = EXCLUDED.color",
        )
        .bind(&event.event_id)
        .bind(&event.schedule_id)
        .bind(&event.title)
        .bind(&event.course_id)
        .bind(&event.course_name)
        .bind(Json(&event.teachers))
        .bind(Json(&event.locations))
        .bind(event.from)
        .bind(event.to)
        .bind(event.last_modified)
        .bind(event.is_special)
        .bind(&event.school_code)
        .bind(&event.color_hex)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await
}

/// All events whose `schedule_id` is in the given set.
///
/// # Errors
/// Returns any error from the query.
pub async fn get_events_by_schedule_ids(
    pool: &PgPool,
    schedule_ids: &[String],
) -> Result<Vec<Event>, sqlx::Error> {
    let rows = sqlx::query("SELECT * FROM events WHERE schedule_id = ANY($1)")
        .bind(schedule_ids)
        .fetch_all(pool)
        .await?;
    Ok(rows.iter().map(row_to_event).collect())
}

/// Distinct (schedule, school) pairs currently stored — the refresh work list.
///
/// # Errors
/// Returns any error from the query.
pub async fn distinct_schedule_refs(pool: &PgPool) -> Result<Vec<ScheduleRef>, sqlx::Error> {
    let rows = sqlx::query("SELECT DISTINCT schedule_id, school_code FROM events")
        .fetch_all(pool)
        .await?;
    Ok(rows
        .iter()
        .map(|row| ScheduleRef {
            schedule_id: row.get("schedule_id"),
            school_code: row.get("school_code"),
        })
        .collect())
}

fn row_to_event(row: &sqlx::postgres::PgRow) -> Event {
    Event {
        event_id: row.get("event_id"),
        schedule_id: row.get("schedule_id"),
        title: row.get("title"),
        course_id: row.get("course_id"),
        course_name: row.get("course_name"),
        teachers: row.get::<Json<Vec<Teacher>>, _>("teachers").0,
        locations: row.get::<Json<Vec<Location>>, _>("locations").0,
        from: row.get("start_at"),
        to: row.get("end_at"),
        last_modified: row.get("last_modified"),
        is_special: row.get("is_special"),
        school_code: row.get("school_code"),
        color_hex: row.get("color"),
    }
}
