use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

const DEFAULT_PORT: u16 = 7077;

pub struct Config {
    pub port: u16,
    pub database_url: Option<String>, // we don't need one, but it will offer cache and background refreshes
    pub log_level: log::LevelFilter,
}

impl Config {
    #[must_use]
    pub fn from_env() -> Self {
        let port = std::env::var("PORT")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(DEFAULT_PORT);
        let database_url = std::env::var("DATABASE_URL")
            .ok()
            .map(|url| url.trim_end_matches('/').to_owned());
        let log_level = std::env::var("LOG_LEVEL")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(log::LevelFilter::Debug);
        Self {
            port,
            database_url,
            log_level,
        }
    }

    /// Build a lazy connection pool if a database is configured.
    ///
    /// # Panics
    /// Panics if `DATABASE_URL` is set but not a valid connection string.
    #[must_use]
    pub fn build_pool(&self) -> Option<PgPool> {
        self.database_url.as_ref().map(|url| {
            PgPoolOptions::new()
                .connect_lazy(url)
                .expect("DATABASE_URL is a valid Postgres connection string")
        })
    }
}
