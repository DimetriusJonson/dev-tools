use std::path::PathBuf;

use axum::body::Body as AxumBody;
use axum::extract::{DefaultBodyLimit, Request, State};
use axum::response::Response as AxumResponse;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, get_service, post};
use axum::{Router, middleware};
use http::{StatusCode, Uri};
use leptos::prelude::*;
use tower::ServiceExt;
use tower_http::compression::CompressionLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

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

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
            <!DOCTYPE html>
            <html lang="en">
                <head>
                    <meta charset="utf-8"/>
                    <meta name="viewport" content="width=device-width, initial-scale=1"/>
                    <AutoReload options=options.clone() />
                    <HydrationScripts options/>
    //                <MetaTags/>
                    <link rel="manifest" href="/manifest.json"/>

                    <script src="/codemirror.min.js"></script>
                </head>
                <body>
                </body>
            </html>
        }
}

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

    let index_path = PathBuf::from(&*leptos_options.site_root).join("index.html");

    tokio::fs::write(index_path.to_owned(), shell(leptos_options.clone()).to_html())
        .await
        .expect("could not write index.html");

    let index_service = get_service(ServeFile::new(index_path));

    let app = Router::new()
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
        .route("/urlEncoder", index_service.clone())
        .route("/json", index_service.clone())
        .route("/share_file", index_service.clone())
        .route("/share_file/view", index_service.clone())
        .route("/compare_text", index_service.clone())
        .route("/rest_client", index_service.clone())
        .route("/rest_client_info", index_service.clone())
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
    match get_static_file(uri.clone(), &root).await {
        Ok(res) => res.into_response(),
        Err(_) => get_static_file(Uri::from_static("/index.html"), &root)
            .await
            .expect("could not find index.html")
            .into_response(),
    }
}

async fn get_static_file(uri: Uri, root: &str) -> Result<Response<AxumBody>, (StatusCode, String)> {
    let req = Request::builder().uri(uri.clone()).body(AxumBody::empty()).unwrap();
    // `ServeDir` implements `tower::Service` so we can call it with `tower::ServiceExt::oneshot`
    // This path is relative to the cargo root
    match ServeDir::new(root).oneshot(req).await {
        Ok(res) => Ok(res.map(AxumBody::new)),
        Err(err) => {
            Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Something went wrong: {err}")))
        }
    }
}
