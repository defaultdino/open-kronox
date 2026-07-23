#[macro_use]
extern crate rocket;

mod config;
mod error;
mod fairings;
mod routes;
mod service;
mod state;

use rocket::{State, http::Status};

use crate::config::Config;
use crate::state::Deps;

#[get("/healthz")]
async fn healthz(deps: &State<Deps>) -> Result<&'static str, Status> {
    match &deps.pool {
        None => Ok("ok"),
        Some(pool) => sqlx::query("SELECT 1")
            .execute(pool)
            .await
            .map(|_| "ok")
            .map_err(|_| Status::ServiceUnavailable),
    }
}

#[catch(404)]
fn not_found() -> &'static str {
    "404 - Not Found"
}

#[catch(500)]
fn internal_error() -> &'static str {
    "500 - Internal Server Error"
}

#[rocket::main]
async fn main() -> Result<(), Box<rocket::Error>> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("failed to install the default rustls crypto provider");

    let config = Config::from_env();
    let schools = kronox::SchoolsConfig::load().expect("failed to load schools config");
    let client = kronox::Client::new().expect("failed to build kronox HTTP client");

    let pool = config.build_pool();
    let has_db = pool.is_some();
    if has_db {
        info!("storage: Postgres cache enabled");
    } else {
        info!("storage: none (stateless scrape-through)");
    }

    let deps = Deps::new(pool, client, schools);

    let figment = rocket::Config::figment()
        .merge(("port", config.port))
        .merge(("address", "0.0.0.0"));

    let mut server = rocket::custom(figment)
        .manage(deps)
        .register("/", catchers![not_found, internal_error])
        .mount("/", routes![healthz])
        .mount(
            "/api/v1",
            routes![
                routes::programme::search,
                routes::schedule::events,
                routes::schedule::rooms,
                routes::schedule::teachers,
                routes::schedule::courses,
                routes::schedule::today,
                routes::schedule::next,
            ],
        );

    if has_db {
        server = server
            .attach(fairings::migrations())
            .attach(fairings::background_refresh());
    }

    server.launch().await.map_err(Box::new)?;
    Ok(())
}
