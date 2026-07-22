use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Teacher {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub id: String,
    pub name: String,
    pub building: String,
    pub floor: String,
    pub max_seats: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    #[serde(rename = "id")]
    pub event_id: String,
    pub schedule_id: String,
    pub title: String,
    pub course_id: String,
    pub course_name: String,
    pub teachers: Vec<Teacher>,
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
    pub locations: Vec<Location>,
    pub last_modified: DateTime<Utc>,
    pub is_special: bool,
    pub school_code: String,
    #[serde(rename = "color")]
    pub color_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Programme {
    pub id: String,
    pub title: String,
    pub subtitle: String,
}
