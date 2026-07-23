use std::time::Duration;

use rocket::fairing::{self, AdHoc};

use crate::service;
use crate::state::Deps;

const BACKGROUND_REFRESH: Duration = Duration::from_hours(6);

#[must_use]
pub fn migrations() -> AdHoc {
    AdHoc::try_on_ignite("migrations", |rocket| Box::pin(run_migrations(rocket)))
}

async fn run_migrations(rocket: rocket::Rocket<rocket::Build>) -> fairing::Result {
    let deps = rocket.state::<Deps>().expect("Deps managed before ignite");
    let pool = deps
        .pool
        .as_ref()
        .expect("migrations run only with a database");
    match storage_pg::MIGRATOR.run(pool).await {
        Ok(()) => Ok(rocket),
        Err(error) => {
            rocket::error!("migrations failed: {error}");
            Err(rocket)
        }
    }
}

/// refresh all cached schedules on startup and every [`BACKGROUND_REFRESH`].
#[must_use]
pub fn background_refresh() -> AdHoc {
    AdHoc::on_liftoff("background-refresh", |rocket| {
        Box::pin(async move {
            let deps = rocket
                .state::<Deps>()
                .expect("Deps managed before liftoff")
                .clone();
            rocket::tokio::spawn(async move {
                let mut ticker = rocket::tokio::time::interval(BACKGROUND_REFRESH);
                loop {
                    ticker.tick().await;
                    service::refresh_all_known(&deps).await;
                }
            });
        })
    })
}
