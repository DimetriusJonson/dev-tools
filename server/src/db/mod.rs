pub mod share_files_db;

use log::info;
use sqlx::{Pool, Postgres};

pub async fn create_pool(database_url: String) -> Pool<Postgres> {
    info!("Connect to database...");
    let pool = sqlx::postgres::PgPoolOptions::new()
        .min_connections(1)
        .max_connections(3)
        .connect(database_url.as_str())
        .await
        .expect("could not connect to database_url");

    //sqlx::migrate!("./migrations").run(&pool).await.expect("migrations failed");

    pool
}
