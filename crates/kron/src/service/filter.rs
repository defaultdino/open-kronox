//! In-memory filtering and projection of events, shared by the schedule
//! endpoints. Pure functions over [`Event`], easy to test in isolation.

use std::collections::HashSet;

use chrono::{DateTime, Utc};
use data_model::{Event, Location, Teacher};
use serde::Serialize;

#[derive(Default)]
pub struct Filter {
    pub room_id: Option<String>,
    pub teacher_id: Option<String>,
    pub course_id: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
pub struct CourseRef {
    pub course_id: String,
    pub course_name: String,
}

#[must_use]
pub fn apply(events: Vec<Event>, filter: &Filter) -> Vec<Event> {
    events
        .into_iter()
        .filter(|event| matches(event, filter))
        .collect()
}

fn matches(event: &Event, filter: &Filter) -> bool {
    if let Some(room_id) = &filter.room_id
        && !event
            .locations
            .iter()
            .any(|location| &location.id == room_id)
    {
        return false;
    }
    if let Some(teacher_id) = &filter.teacher_id
        && !event
            .teachers
            .iter()
            .any(|teacher| &teacher.id == teacher_id)
    {
        return false;
    }
    if let Some(course_id) = &filter.course_id
        && &event.course_id != course_id
    {
        return false;
    }
    if let Some(from) = filter.from
        && event.from < from
    {
        return false;
    }
    if let Some(to) = filter.to
        && event.from >= to
    {
        return false;
    }
    true
}

#[must_use]
pub fn distinct_locations(events: &[Event]) -> Vec<Location> {
    let mut seen = HashSet::new();
    events
        .iter()
        .flat_map(|event| &event.locations)
        .filter(|location| !location.id.is_empty() && seen.insert(location.id.clone()))
        .cloned()
        .collect()
}

#[must_use]
pub fn distinct_teachers(events: &[Event]) -> Vec<Teacher> {
    let mut seen = HashSet::new();
    events
        .iter()
        .flat_map(|event| &event.teachers)
        .filter(|teacher| !teacher.id.is_empty() && seen.insert(teacher.id.clone()))
        .cloned()
        .collect()
}

#[must_use]
pub fn distinct_courses(events: &[Event]) -> Vec<CourseRef> {
    let mut seen = HashSet::new();
    events
        .iter()
        .filter(|event| !event.course_id.is_empty() && seen.insert(event.course_id.clone()))
        .map(|event| CourseRef {
            course_id: event.course_id.clone(),
            course_name: event.course_name.clone(),
        })
        .collect()
}

#[must_use]
pub fn sort_by_start(mut events: Vec<Event>) -> Vec<Event> {
    events.sort_by_key(|event| event.from);
    events
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at(hour: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 7, 22, hour, 0, 0).unwrap()
    }

    fn event(id: &str, course: &str, start_hour: u32) -> Event {
        Event {
            event_id: id.to_owned(),
            schedule_id: "sched".to_owned(),
            title: "t".to_owned(),
            course_id: course.to_owned(),
            course_name: format!("name-{course}"),
            teachers: vec![Teacher {
                id: "teach1".to_owned(),
                first_name: "a".to_owned(),
                last_name: "b".to_owned(),
            }],
            from: at(start_hour),
            to: at(start_hour + 1),
            locations: vec![Location {
                id: "room1".to_owned(),
                name: "n".to_owned(),
                building: "x".to_owned(),
                floor: "1".to_owned(),
                max_seats: "10".to_owned(),
            }],
            last_modified: at(0),
            is_special: false,
            school_code: "hkr".to_owned(),
            color_hex: "#4A90E2".to_owned(),
        }
    }

    #[test]
    fn from_inclusive_to_exclusive() {
        let events = vec![
            event("a", "c1", 8),
            event("b", "c1", 10),
            event("c", "c2", 12),
        ];
        let filter = Filter {
            from: Some(at(10)),
            to: Some(at(12)),
            ..Filter::default()
        };
        let out = apply(events, &filter);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].event_id, "b");
    }

    #[test]
    fn distinct_dedupes_by_id() {
        let events = vec![event("a", "c1", 8), event("b", "c1", 10)];
        assert_eq!(distinct_courses(&events).len(), 1);
        assert_eq!(distinct_locations(&events).len(), 1);
        assert_eq!(distinct_teachers(&events).len(), 1);
    }
}
