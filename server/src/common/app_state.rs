use axum::extract::FromRef;
use leptos::prelude::LeptosOptions;
use sqlx::{Pool, Postgres};

#[derive(FromRef, Debug, Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub pool: Option<Pool<Postgres>>,
}
