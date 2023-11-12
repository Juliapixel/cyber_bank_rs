CREATE TABLE if not exists users (
    user_id uuid NOT NULL PRIMARY KEY,
    email text NOT NULL UNIQUE,
    username text NOT NULL UNIQUE,
    password bytea NOT NULL,
    salt bytea NOT NULL,
    creation_date timestamp with time zone NOT NULL
);

CREATE TABLE if not exists revoked_tokens (
    token text NOT NULL UNIQUE,
    expiration_date timestamp with time zone NOT NULL
);
