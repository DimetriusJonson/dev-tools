use std::net::SocketAddr;

use axum::extract::FromRef;
use sqlx::{Pool, Postgres};

#[derive(FromRef, Debug, Clone)]
pub struct AppState {
    pub addr: SocketAddr,
    pub pool: Option<Pool<Postgres>>,
    pub remote_server_url: Option<String>,
}
