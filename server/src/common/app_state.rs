use axum::extract::FromRef;
use leptos::prelude::LeptosOptions;

use crate::db::DbPool;

#[derive(FromRef, Debug, Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub pool: Option<DbPool>,
    pub remote_server_url: Option<String>,
    pub dump_port: u16,
    pub max_content_length: u64,
    pub rest_client_proxy_allow_ips: Vec<String>,
}
