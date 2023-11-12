# cyber_bank_rs

## Status

- [x] user registration and authentication endpoints
- [ ] token-based session management (possibly using redis?)
- [ ] bank account management endpoints
- [ ] money transfer endpoints
- [ ] (fake) currency exchange with simulated value fluctuations
- [ ] (maybe) stock exchange simulation (maybe) using real live data
- [ ] a web UI for all of the above items

not planned:
- native UIs (mobile, dekstop)

## Available API endpoints

### `/auth/register`

#### example request:

Headers:
- Content-type: application/json

Content:

```json
{
    "email": "{email}",
    "username": "{username}",
    "password": "{password}"
}
```

##### on success:

HTTP Status 201

##### on failure:

HTTP Status 400

Content:

```json
{
    ["{failure_point}": "{failure_reason}"]
}
```

### `/auth/login`

#### example request:

Headers:
- Content-type: application/json

Content:
```json
{
    "username": "{username}",
    "password": "{password}"
}
```

##### on success:

HTTP Status 200

Content:
```json
{
    "token": "{token}"
}
```

##### on failure:

HTTP Status 403


## Building
building requires a connection to a PostgreSQL database with the correct relations set up.

**⚠️ only tested on linux**

### Setting up development database
1. start up a PostgreSQL server
2. create a database called `cyber_bank_rs`
3. create a `.env` file in the project root with `DATABASE_URL={url_to_database}` in it
4. install `sqlx-cli` by running `cargo install sqlx`
5. run `sqlx migrate run`

after that, just run `cargo build --release --locked`

## Running

1. add the following variables to your `.env` file:
    - `POSTGRES_HOST={database_hostname}`
    - `POSTGRES_PORT={database_port}`
    - `POSTGRES_PASSWORD={database_password}`
2. run `cargo run --release --locked --bin server`
