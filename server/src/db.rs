use log::info;
use sqlx::{Pool, Postgres};

pub async fn create_pool() -> Option<Pool<Postgres>> {
    let database_url = std::env::var("DATABASE_URL");
    match database_url {
        Ok(database_url) => {
            info!("Connect to database...");
            let pool = sqlx::postgres::PgPoolOptions::new()
                .min_connections(1)
                .max_connections(3)
                .connect(database_url.as_str())
                .await
                .expect("could not connect to database_url");

            //sqlx::migrate!("./migrations").run(&pool).await.expect("migrations failed");

            Some(pool)
        }
        Err(_) => {
            info!("Database not found!");
            None 
        },
    }
}
