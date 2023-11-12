CREATE TABLE if not exists users (
    user_id uuid NOT NULL PRIMARY KEY,
    email text NOT NULL UNIQUE,
    username text NOT NULL UNIQUE,
    password bytea NOT NULL,
    salt bytea NOT NULL,
    creation_date timestamp with time zone NOT NULL
);

CREATE INDEX users_username ON users USING HASH (username);
CREATE INDEX users_email ON users USING HASH (email);

CREATE TABLE if not exists revoked_tokens (
    token text NOT NULL UNIQUE,
    expiration_date timestamp with time zone NOT NULL
);

CREATE INDEX revoked_tokens_token ON revoked_tokens USING HASH (token);
CREATE INDEX revoked_tokens_expiration_date ON revoked_tokens USING BTREE (expiration_date);
