CREATE TABLE events (
    event_id      text PRIMARY KEY,
    schedule_id   text NOT NULL,
    title         text NOT NULL,
    course_id     text NOT NULL,
    course_name   text NOT NULL,
    teachers      jsonb NOT NULL,
    locations     jsonb NOT NULL,
    start_at      timestamptz NOT NULL,
    end_at        timestamptz NOT NULL,
    last_modified timestamptz NOT NULL,
    is_special    boolean NOT NULL,
    school_code   text NOT NULL,
    color         text NOT NULL
);

CREATE INDEX events_schedule_id_idx ON events (schedule_id);

CREATE TABLE schedule_meta (
    schedule_id  text PRIMARY KEY,
    school_code  text NOT NULL,
    last_updated timestamptz NOT NULL
);
