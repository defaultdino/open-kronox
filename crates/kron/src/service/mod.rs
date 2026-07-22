//! business logic (fetching, caching, and filtering schedules and programmes)

pub mod filter;
mod programme;
mod schedule;

pub use programme::search as search_programmes;
pub use schedule::{query_events, refresh_all_known};

#[derive(Debug)]
pub enum ServiceError {
    Upstream, // kronox problem
    Db,       // our problem
}
