CREATE TABLE if not exists users (
    id bigserial NOT NULL UNIQUE PRIMARY KEY,
    user_id uuid NOT NULL,
    email text NOT NULL UNIQUE,
    username text NOT NULL UNIQUE,
    password bytea NOT NULL,
    salt bytea NOT NULL
);
