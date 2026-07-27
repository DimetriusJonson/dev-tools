use std::net::SocketAddr;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post};
use sqlx::{Pool, Postgres};
use tower_http::compression::CompressionLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use crate::app_router::json_format_router::format_json_handler;
use crate::app_router::rest_client_router::rest_client_send_handler;
use crate::app_router::share_file_router::{
    share_file_custom_servers_handler, share_file_download, share_file_info,
    share_file_info_ex_handler, share_file_upload,
};
use crate::app_router::share_local_file_router::{
    share_local_file_download, share_local_file_info, share_local_file_upload,
};
use crate::app_router::test_json_router::test_json_handler;
use crate::app_router::xml_format_router::format_xml_handler;
use crate::common::app_state::AppState;

pub async fn build_app_router(
    addr: SocketAddr,
    pool: Option<Pool<Postgres>>,
    remote_server_url: Option<String>,
    dist_dir: String,
) -> anyhow::Result<Router> {
    let app_state = AppState { addr, pool: pool.clone(), remote_server_url };

    let app = Router::new()
        .route("/rest_client_send", post(rest_client_send_handler))
        .route("/format_xml", post(format_xml_handler))
        .route("/format_json", post(format_json_handler))
        .route("/share_local_file_upload", post(share_local_file_upload))
        .route("/share_file_upload", post(share_file_upload))
        .layer(DefaultBodyLimit::disable())
        .route("/share_file_download", get(share_file_download))
        .route("/share_file_info", get(share_file_info))
        .route("/share_file_custom_servers", get(share_file_custom_servers_handler))
        .route("/share_file_info_ex", get(share_file_info_ex_handler))
        .route("/share_local_file_info", get(share_local_file_info))
        .route("/share_local_file_download", get(share_local_file_download))
        .route("/test_json", get(test_json_handler))
        .fallback_service(ServeDir::new(dist_dir))
        .layer(CompressionLayer::new().gzip(true))
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    Ok(app)
}
