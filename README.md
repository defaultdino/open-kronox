# open-kronox

A small, self-contained HTTP API over [KronoX](https://www.kronox.se/), the
scheduling system used by several Swedish universities. It scrapes a school's
KronoX instance and serves clean JSON: schedule events, room/teacher/course
listings, and programme search.

Written in Rust as a single binary. Using a database is optional.

> **Unofficial.** This project is not affiliated with KronoX or any university.
> It parses public KronoX pages. You are responsible for your own use, including
> request volume and each site's terms. Put it behind a reverse proxy and add
> rate limiting before exposing it publicly.

## Quick start

```sh
cp schools.example.json schools.json
# edit schools.json to taste, then:
KRONOX_SCHOOLS_FILE=schools.json cargo run -p kron
```

If you want to set the application log level feel free to do so using the `LOG_LEVEL` environment variable, where applicable values include `info`, `debug`, `warn`, `off`, `error` and `trace`.

```
export KRONOX_SCHOOLS_FILE=schools.json
export LOG_LEVEL=debug

cargo run -p kron
```

Then:

```sh
curl 'http://localhost:7077/api/v1/programme/search?school=hkr&search_query=data'
curl 'http://localhost:7077/api/v1/schedule/events?school=hkr&schedule_ids=<id>'
```

## Endpoints

All under `/api/v1`. `school` is required everywhere; `schedule_ids` is a
comma-separated list required by the schedule endpoints.

| Endpoint                 | Purpose                        | Extra params                                                 |
| ------------------------ | ------------------------------ | ------------------------------------------------------------ |
| `GET /programme/search`  | Free-text programme search     | `search_query`                                               |
| `GET /schedule/events`   | Events for the given schedules | `room_id`, `teacher_id`, `course_id`, `from`, `to` (RFC3339) |
| `GET /schedule/rooms`    | Distinct rooms in those events | same filters                                                 |
| `GET /schedule/teachers` | Distinct teachers              | same filters                                                 |
| `GET /schedule/courses`  | Distinct courses               | same filters                                                 |
| `GET /schedule/today`    | Today's events, sorted         | `tz` (IANA, default `Europe/Stockholm`)                      |
| `GET /schedule/next`     | Next N upcoming events         | `n` (default 5, max 50)                                      |
| `GET /healthz`           | Liveness                       | —                                                            |

Filters are AND-combined; `from` is inclusive, `to` is exclusive on the event
start time.

## Configuration

| Env var               | Required     | Default                    | Meaning                                |
| --------------------- | ------------ | -------------------------- | -------------------------------------- |
| `KRONOX_SCHOOLS_JSON` | one of these | —                          | Schools config as inline JSON          |
| `KRONOX_SCHOOLS_FILE` | for schools  | `.well-known/schools.json` | Path to schools config                 |
| `PORT`                | no           | `7077`                     | Listen port                            |
| `DATABASE_URL`        | no           | —                          | Enables the Postgres cache (see below) |

A school code maps to one or more KronoX base URLs; requests fail over to the
next URL if one is down. See `schools.example.json`.

## Optional Postgres cache

Set `DATABASE_URL` to a Postgres connection string and the server switches from
stateless scrape-through to using a cache: scraped events are stored, repeat requests
are served from Postgres, and stale schedules (older than 6h) refresh in the
background. Migrations run automatically on startup. This is all optional.

## Development

```sh
cargo test
cargo clippy --workspace
```

The reusable scraper lives in the `kronox` crate (`crates/kronox`) and has no
database dependency, if you want the KronoX client without the HTTP server.

## License

Apache-2.0. See [LICENSE](LICENSE).
