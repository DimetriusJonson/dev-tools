use app::app::{App, shell};
use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::routing::post;
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list};
use tower_http::compression::CompressionLayer;
use tower_http::trace::TraceLayer;

use crate::app_router::json_escape_router::unescape_json_handler;
use crate::app_router::json_format_router::format_json_handler;
use crate::app_router::url_encode_router::{decode_url_handler, encode_url_handler};
use crate::app_router::xml_format_router::format_xml_handler;
use crate::app_router::xml_escape_router::{escape_xml_handler, unescape_xml_handler};

/* ========================================================== */
/*                         🦀 MAIN 🦀                         */
/* ========================================================== */

pub async fn build_app_router(conf_file: ConfFile) -> anyhow::Result<Router> {
    let leptos_options = conf_file.leptos_options;

    let routes = generate_route_list(App);

    let app = Router::new()
        .route("/format_xml", post(format_xml_handler))
        .route("/unescape_xml", post(unescape_xml_handler))
        .route("/escape_xml", post(escape_xml_handler))
        .route("/format_json", post(format_json_handler))
        .layer(DefaultBodyLimit::disable())
        .route("/encode_url", post(encode_url_handler))
        .route("/decode_url", post(decode_url_handler))
        .route("/unescape_json", post(unescape_json_handler))
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .layer(CompressionLayer::new().gzip(true))
        .layer(TraceLayer::new_for_http())
        .with_state(leptos_options);

    Ok(app)
}