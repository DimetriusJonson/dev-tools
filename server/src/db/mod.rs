pub mod share_files_db;

#[cfg(feature = "sharefiledb")]
use sqlx::{Pool, Postgres};

#[cfg(feature = "sharefiledb")]
pub type DbPool = Pool<Postgres>;

#[cfg(not(feature = "sharefiledb"))]
pub type DbPool = ();

#[cfg(feature = "sharefiledb")]
pub async fn create_pool(database_url: String) -> DbPool {
    use tracing::info;

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
