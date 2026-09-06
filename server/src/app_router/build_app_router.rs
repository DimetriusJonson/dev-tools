use axum::body::Body as AxumBody;
use axum::extract::{DefaultBodyLimit, Request, State};
use axum::response::Response as AxumResponse;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Router, middleware};
use http::{StatusCode, Uri};
use leptos::prelude::*;
use tower::ServiceExt;
use tower_http::compression::CompressionLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use crate::app_router::index_router::index_handler;
use crate::app_router::json_format_router::format_json_handler;
use crate::app_router::rest_client_router::{
    rest_client_attachment_download_handler, rest_client_html_previewer_middleware,
    rest_client_proxy_allow, rest_client_send_handler,
};
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
use crate::db::DbPool;

pub async fn build_app_router(
    conf_file: ConfFile,
    pool: Option<DbPool>,
    remote_server_url: Option<String>,
    dump_port: u16,
    rc_max_content_length: u64,
    rest_client_proxy_allow_ips: Vec<String>,
) -> anyhow::Result<Router> {
    let leptos_options = conf_file.leptos_options;

    let app_state = AppState {
        leptos_options: leptos_options.clone(),
        pool: pool.clone(),
        remote_server_url,
        dump_port,
        max_content_length: rc_max_content_length,
        rest_client_proxy_allow_ips,
    };

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/index.html", get(index_handler))
        .route("/rest_client_send", post(rest_client_send_handler))
        .route("/rest_client_proxy_allow", get(rest_client_proxy_allow))
        .route("/rest_client_attachment_download", post(rest_client_attachment_download_handler))
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
        .route("/urlEncoder", get(index_handler))
        .route("/json", get(index_handler))
        .route("/share_file", get(index_handler))
        .route("/share_file/view", get(index_handler))
        .route("/compare_text", get(index_handler))
        .route("/rest_client", get(index_handler))
        .route("/rest_client_info", get(index_handler))
        .fallback(file_and_error_handler)
        .layer(CompressionLayer::new().gzip(true))
        .layer(TraceLayer::new_for_http())
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            move |app_state, connect_info, req, next| {
                rest_client_html_previewer_middleware(
                    app_state,
                    connect_info,
                    vec![
                        "/urlEncoder".to_owned(),
                        "/json".to_owned(),
                        "/share_file".to_owned(),
                        "/share_file/view".to_owned(),
                        "/compare_text".to_owned(),
                        "/rest_client".to_owned(),
                        "/rest_client_info".to_owned(),
                    ],
                    req,
                    next,
                )
            },
        ))
        .with_state(app_state);

    Ok(app)
}

pub async fn file_and_error_handler(
    uri: Uri,
    State(options): State<LeptosOptions>,
) -> AxumResponse {
    let root = options.site_root.clone();
    get_static_file(uri.clone(), &root).await.into_response()
}

async fn get_static_file(uri: Uri, root: &str) -> Result<Response<AxumBody>, (StatusCode, String)> {
    let req = Request::builder().uri(uri.clone()).body(AxumBody::empty()).unwrap();
    match ServeDir::new(root).oneshot(req).await {
        Ok(res) => Ok(res.map(AxumBody::new)),
        Err(err) => {
            Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Something went wrong: {err}")))
        }
    }
}
