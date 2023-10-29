use actix_web::{HttpServer, App};
use sqlx::postgres::PgConnectOptions;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(
        env_logger::Env::default()
            .default_filter_or("DEBUG")
    );

    let pg_host = std::env::var("POSTGRES_HOST")
        .expect("POSTGRES_HOST ENV VAR NOT SET");
    let pg_port = std::env::var("POSTGRES_PORT")
        .expect("POSTGRES_PORT ENV VAR NOT SET");
    let pg_pass = std::env::var("POSTGRES_PASSWORD")
        .expect("POSTGRES_PASSWORD ENV VAR NOT SET");

    let db_connection = sqlx::PgPool::connect_with(
        PgConnectOptions::new()
            .host(&pg_host)
            .port(pg_port.parse().unwrap())
            .username("postgres")
            .password(&pg_pass)
            .database("cyber_bank_rs")
    ).await.unwrap();

    sqlx::query_file!("./migrations/setup.sql")
        .fetch_all(&db_connection)
        .await
        .unwrap();

    HttpServer::new(move || {
        let conn = db_connection.clone();
        App::new()
            .configure(cyber_bank_rs::auth::config)
            .app_data(conn)
    }).bind(("0.0.0.0", 8080))
        .unwrap()
        .run()
        .await
}
