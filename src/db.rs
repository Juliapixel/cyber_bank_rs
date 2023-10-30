use sqlx::{postgres::PgConnectOptions, PgPool};

pub async fn get_db_pool() -> PgPool {
    let pg_host = dotenvy::var("POSTGRES_HOST")
        .expect("POSTGRES_HOST ENV VAR NOT SET");
    let pg_port = dotenvy::var("POSTGRES_PORT")
        .expect("POSTGRES_PORT ENV VAR NOT SET");
    let pg_pass = dotenvy::var("POSTGRES_PASSWORD")
        .expect("POSTGRES_PASSWORD ENV VAR NOT SET");

    let db_connection = PgPool::connect_with(
        PgConnectOptions::new()
            .host(&pg_host)
            .port(pg_port.parse().unwrap())
            .username("postgres")
            .password(&pg_pass)
            .database("cyber_bank_rs")
    ).await.unwrap();
    return db_connection;
}

pub async fn set_up_db_tables(pool: &PgPool) {
    sqlx::query_file!("./migrations/setup.sql")
        .fetch_all(pool)
        .await
        .unwrap();
}
