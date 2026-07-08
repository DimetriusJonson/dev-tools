use axum::extract::FromRef;
use leptos::prelude::LeptosOptions;
use reqwest::Client;
use sqlx::{Pool, Postgres};

#[derive(FromRef, Debug, Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub pool: Option<Pool<Postgres>>,
    pub client: Option<Client>,
    pub remote_server_url: Option<String>,
}
