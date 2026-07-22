use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use kronox::{Client, SchoolsConfig};
use sqlx::PgPool;

pub type RefreshLocks = Arc<Mutex<HashSet<String>>>;

#[derive(Clone)]
pub struct Deps {
    pub pool: Option<PgPool>,
    pub client: Client,
    pub schools: SchoolsConfig,
    pub locks: RefreshLocks,
}

impl Deps {
    #[must_use]
    pub fn new(pool: Option<PgPool>, client: Client, schools: SchoolsConfig) -> Self {
        Self {
            pool,
            client,
            schools,
            locks: Arc::new(Mutex::new(HashSet::new())),
        }
    }
}
