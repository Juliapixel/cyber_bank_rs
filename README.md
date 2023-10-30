# cyber_bank_rs

## Building
building requires a connection to a PostgreSQL database with the correct relations set up.

### Setting up development database
1. start up a PostgreSQL server
2. create a database called `cyber_bank_rs`
3. run the commands specified in [setup.sql](/migrations/setup.sql)
4. create a `.env` file in the project root with `DATABASE_URL={url_to_database}` in it

after that, just run `cargo build --release --locked`
