
use axum::body::Body as AxumBody;
use app::app::{App, shell};
use axum::Router;
use axum::extract::{DefaultBodyLimit, Request, State};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use leptos::config::ConfFile;
use leptos::context::provide_context;
use leptos_axum::{LeptosRoutes, generate_route_list, handle_server_fns_with_context, render_app_to_stream_with_context};
use sqlx::{Pool, Postgres};
use tower_http::compression::CompressionLayer;
use tower_http::trace::TraceLayer;

use crate::app_router::json_format_router::format_json_handler;
use crate::app_router::rest_client_router::{rest_client_attachment_download_handler, rest_client_send_handler};
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
    conf_file: ConfFile,
    pool: Option<Pool<Postgres>>,
    remote_server_url: Option<String>,
    dump_port: u16,
    rc_max_content_length: u64,
) -> anyhow::Result<Router> {
    let leptos_options = conf_file.leptos_options;

    let routes = generate_route_list(App);

    let app_state =
        AppState { leptos_options: leptos_options.clone(), pool: pool.clone(), remote_server_url, dump_port, max_content_length: rc_max_content_length };

    let app = Router::new()
        .route("/rest_client_send", post(rest_client_send_handler))
        .route("/rest_client_attachment_download", post(rest_client_attachment_download_handler))
//        .route("/rest_client_get_url", get(rest_client_get_url_handler))
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
        .route("/api/{*fn_name}", get(server_fn_handler).post(server_fn_handler))
        .leptos_routes_with_handler(routes, get(leptos_routes_handler))
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>(shell))
        .layer(CompressionLayer::new().gzip(true))
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    Ok(app)
}

#[axum_macros::debug_handler]
pub async fn leptos_routes_handler(
    State(app_state): State<AppState>,
    req: Request<AxumBody>,
) -> Response {
    let leptos_options = app_state.leptos_options.clone();

    let handler = render_app_to_stream_with_context(
        move || provide_context(app_state.clone()),
        move || shell(leptos_options.clone()),
    );
    handler(req).await.into_response()
}

#[axum_macros::debug_handler]
pub async fn server_fn_handler(
    State(state): State<AppState>,
    request: Request<AxumBody>,
) -> impl IntoResponse {
    handle_server_fns_with_context(
        move || {
            provide_context(state.clone());
        },
        request,
    )
    .await
}
