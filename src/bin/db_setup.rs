use cyber_bank_rs::db;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(
        env_logger::Env::default()
            .default_filter_or("DEBUG")
    );
    db::create_database().await;

    let pool = db::get_db_pool().await;

    // sets up tables and stuff for the database (in case it wasn't already set up)
    db::set_up_db_tables(&pool).await;

    Ok(())
}
