use chrono::{DateTime, Duration, TimeZone, Utc};
use chrono_tz::Tz;
use rocket::State;
use rocket::serde::json::{Json, json};

use crate::error::ApiError;
use crate::routes::{ApiResult, require_school, split_schedule_ids};
use crate::service::{self, filter::Filter};
use crate::state::Deps;

const DEFAULT_TZ: &str = "Europe/Stockholm";
const DEFAULT_NEXT: usize = 5;
const MAX_NEXT: usize = 50;

#[derive(FromForm)]
pub struct ScheduleQuery {
    schedule_ids: Option<String>,
    school: Option<String>,
    room_id: Option<String>,
    teacher_id: Option<String>,
    course_id: Option<String>,
    from: Option<String>,
    to: Option<String>,
}

struct ScheduleInput {
    schedule_ids: Vec<String>,
    school: String,
    filter: Filter,
}

fn parse(deps: &Deps, query: ScheduleQuery) -> Result<ScheduleInput, ApiError> {
    let school = require_school(deps, query.school.as_deref())?;
    let schedule_ids = split_schedule_ids(query.schedule_ids.as_deref())?;
    let filter = Filter {
        room_id: query.room_id,
        teacher_id: query.teacher_id,
        course_id: query.course_id,
        from: parse_rfc3339(query.from.as_deref(), "from")?,
        to: parse_rfc3339(query.to.as_deref(), "to")?,
    };
    Ok(ScheduleInput {
        schedule_ids,
        school,
        filter,
    })
}

fn parse_rfc3339(value: Option<&str>, field: &str) -> Result<Option<DateTime<Utc>>, ApiError> {
    match value {
        None => Ok(None),
        Some(raw) => DateTime::parse_from_rfc3339(raw)
            .map(|dt| Some(dt.with_timezone(&Utc)))
            .map_err(|_| ApiError::BadRequest(format!("invalid {field}: expected RFC3339"))),
    }
}

#[get("/schedule/events?<query..>")]
pub async fn events(deps: &State<Deps>, query: ScheduleQuery) -> ApiResult {
    let input = parse(deps, query)?;
    let events =
        service::query_events(deps, &input.schedule_ids, &input.school, &input.filter).await?;
    let count = events.len();
    Ok(Json(json!({ "events": events, "count": count })))
}

#[get("/schedule/rooms?<query..>")]
pub async fn rooms(deps: &State<Deps>, query: ScheduleQuery) -> ApiResult {
    let input = parse(deps, query)?;
    let events =
        service::query_events(deps, &input.schedule_ids, &input.school, &input.filter).await?;
    let rooms = service::filter::distinct_locations(&events);
    let count = rooms.len();
    Ok(Json(json!({ "rooms": rooms, "count": count })))
}

#[get("/schedule/teachers?<query..>")]
pub async fn teachers(deps: &State<Deps>, query: ScheduleQuery) -> ApiResult {
    let input = parse(deps, query)?;
    let events =
        service::query_events(deps, &input.schedule_ids, &input.school, &input.filter).await?;
    let teachers = service::filter::distinct_teachers(&events);
    let count = teachers.len();
    Ok(Json(json!({ "teachers": teachers, "count": count })))
}

#[get("/schedule/courses?<query..>")]
pub async fn courses(deps: &State<Deps>, query: ScheduleQuery) -> ApiResult {
    let input = parse(deps, query)?;
    let events =
        service::query_events(deps, &input.schedule_ids, &input.school, &input.filter).await?;
    let courses = service::filter::distinct_courses(&events);
    let count = courses.len();
    Ok(Json(json!({ "courses": courses, "count": count })))
}

#[get("/schedule/today?<schedule_ids>&<school>&<tz>")]
pub async fn today(
    deps: &State<Deps>,
    schedule_ids: Option<&str>,
    school: Option<&str>,
    tz: Option<&str>,
) -> ApiResult {
    let school = require_school(deps, school)?;
    let schedule_ids = split_schedule_ids(schedule_ids)?;
    let tz_name = tz.unwrap_or(DEFAULT_TZ);
    let zone: Tz = tz_name
        .parse()
        .map_err(|_| ApiError::BadRequest(format!("invalid tz: {tz_name}")))?;

    let (start, end) = day_window(zone);
    let filter = Filter {
        from: Some(start),
        to: Some(end),
        ..Filter::default()
    };
    let events = service::query_events(deps, &schedule_ids, &school, &filter).await?;
    let events = service::filter::sort_by_start(events);
    let count = events.len();
    Ok(Json(
        json!({ "events": events, "count": count, "tz": tz_name }),
    ))
}

#[get("/schedule/next?<schedule_ids>&<school>&<n>")]
pub async fn next(
    deps: &State<Deps>,
    schedule_ids: Option<&str>,
    school: Option<&str>,
    n: Option<usize>,
) -> ApiResult {
    let school = require_school(deps, school)?;
    let schedule_ids = split_schedule_ids(schedule_ids)?;
    let limit = match n {
        Some(value) if (1..=MAX_NEXT).contains(&value) => value,
        _ => DEFAULT_NEXT,
    };

    let filter = Filter {
        from: Some(Utc::now()),
        ..Filter::default()
    };
    let events = service::query_events(deps, &schedule_ids, &school, &filter).await?;
    let mut events = service::filter::sort_by_start(events);
    events.truncate(limit);
    let count = events.len();
    Ok(Json(json!({ "events": events, "count": count })))
}

fn day_window(zone: Tz) -> (DateTime<Utc>, DateTime<Utc>) {
    let today = Utc::now().with_timezone(&zone).date_naive();
    let start_local = today.and_hms_opt(0, 0, 0).expect("midnight is valid");
    let start = zone
        .from_local_datetime(&start_local)
        .single()
        .unwrap_or_else(|| zone.from_utc_datetime(&start_local));
    let start_utc = start.with_timezone(&Utc);
    (start_utc, start_utc + Duration::days(1))
}
