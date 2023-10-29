use actix_web::{HttpServer, App, web};
use sqlx::{postgres::PgConnectOptions, PgPool};

async fn get_db_pool() -> PgPool {
    let pg_host = std::env::var("POSTGRES_HOST")
        .expect("POSTGRES_HOST ENV VAR NOT SET");
    let pg_port = std::env::var("POSTGRES_PORT")
        .expect("POSTGRES_PORT ENV VAR NOT SET");
    let pg_pass = std::env::var("POSTGRES_PASSWORD")
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

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(
        env_logger::Env::default()
            .default_filter_or("DEBUG")
    );

    let pool = get_db_pool().await;

    // sets up tables and stuff for the database (in case it wasn't already set up)
    sqlx::query_file!("./migrations/setup.sql")
        .fetch_all(&pool)
        .await
        .unwrap();

    HttpServer::new(move || {
        let conn = pool.clone();
        App::new()
            // authentication endpoints
            .service(
                web::scope("/auth")
                    .service(
                        // yeah, there's only one version, so what
                        web::scope("/v1")
                            .configure(cyber_bank_rs::auth::config)
                    )
                    // should be set to latest version
                    .configure(cyber_bank_rs::auth::config)
            )
            .app_data(conn)
    }).bind(("0.0.0.0", 8080))
        .unwrap()
        .run()
        .await
}
