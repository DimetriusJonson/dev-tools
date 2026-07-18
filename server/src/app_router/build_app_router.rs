use app::app::{App, shell};
use app::common::app_state::ssr::AppState;
use axum::Router;
use axum::body::Body as AxumBody;
use axum::extract::{DefaultBodyLimit, State};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use http::Request;
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list, handle_server_fns_with_context, render_app_to_stream_with_context};
use sqlx::{Pool, Postgres};
use tower_http::compression::CompressionLayer;
use tower_http::trace::TraceLayer;

use crate::app_router::compare_text_router::compare_text_handler;
use crate::app_router::json_escape_router::{escape_json_handler, unescape_json_handler};
use crate::app_router::json_format_router::format_json_handler;
use crate::app_router::share_file_router::{
    share_file_download, share_file_info, share_file_upload,
};
use crate::app_router::share_local_file_router::{share_local_file_download, share_local_file_info, share_local_file_upload};
use crate::app_router::url_encode_router::{decode_url_handler, encode_url_handler};
use crate::app_router::xml_escape_router::{escape_xml_handler, unescape_xml_handler};
use crate::app_router::xml_format_router::format_xml_handler;

pub async fn build_app_router(
    conf_file: ConfFile,
    pool: Option<Pool<Postgres>>,
    remote_server_url: Option<String>,
) -> anyhow::Result<Router> {
    let leptos_options = conf_file.leptos_options;

    let routes = generate_route_list(App);

    let app_state = AppState {
        leptos_options: leptos_options.clone(),
        pool: pool.clone(),
        remote_server_url,
    };

    let app = Router::new()
        .route("/format_xml", post(format_xml_handler))
        .route("/unescape_xml", post(unescape_xml_handler))
        .route("/escape_xml", post(escape_xml_handler))
        .route("/format_json", post(format_json_handler))
        .route("/escape_json", post(escape_json_handler))
        .route("/unescape_json", post(unescape_json_handler))
        .route("/share_local_file_upload", post(share_local_file_upload))
        .route("/share_file_upload", post(share_file_upload))
        .layer(DefaultBodyLimit::disable())
        .route("/compare_text", post(compare_text_handler))
        .route("/encode_url", post(encode_url_handler))
        .route("/decode_url", post(decode_url_handler))
        .route("/share_file_download", get(share_file_download))
        .route("/share_file_info", get(share_file_info))
        .route("/share_local_file_info", get(share_local_file_info))
        .route("/share_local_file_download", get(share_local_file_download))
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
