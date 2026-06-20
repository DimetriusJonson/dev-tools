use app::app::{App, shell};
use axum::Router;
use axum::routing::post;
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list};
use tower_http::compression::CompressionLayer;
use tower_http::trace::TraceLayer;

use crate::app_router::xml_router::format_xml_handler;

/* ========================================================== */
/*                         🦀 MAIN 🦀                         */
/* ========================================================== */

pub async fn build_app_router(conf_file: ConfFile) -> anyhow::Result<Router> {
    let leptos_options = conf_file.leptos_options;

    let routes = generate_route_list(|| view! { <App /> });

    let app = Router::new()
        .route("/format_xml", post(format_xml_handler))
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
