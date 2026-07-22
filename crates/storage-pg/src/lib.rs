mod error;
mod events;
mod schedule_meta;

pub use error::{is_foreign_key_violation, is_unique_violation, sql_state};
pub use events::{ScheduleRef, distinct_schedule_refs, get_events_by_schedule_ids, upsert_events};
pub use schedule_meta::{get_last_updated, set_last_updated};

pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");
