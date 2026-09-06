use leptos::prelude::*;
use std::{
    ops::Deref,
    sync::{LazyLock, RwLock},
};

use axum::{
    body::Body,
    extract::State,
    response::{IntoResponse, Response},
};
use http::StatusCode;
use leptos::{
    IntoView,
    config::LeptosOptions,
    hydration::{AutoReload, HydrationScripts},
    view,
};

use crate::common::{app_error::AppError, app_state::AppState};

fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <link rel="manifest" href="/manifest.json"/>
                <link rel="stylesheet" href="/pkg/dev_tools.css"/>

                <script src="/codemirror.min.js"></script>
            </head>
            <body class="bg-white dark:bg-dark-bg">
                <div class="flex flex-col items-center justify-center min-h-screen w-full dark:text-white">
                    <div class="flex flex-col items-center space-y-4">
                        <div class="h-12 w-12 animate-spin rounded-full border-4 border-slate-200 border-t-primary dark:border-slate-700 dark:border-t-primary"></div>

                        <p class="text-sm font-medium tracking-wide animate-pulse">
                            Loading application...
                        </p>
                    </div>
                </div>
            </body>
        </html>
    }
}

static INDEX_HTML: LazyLock<RwLock<Box<String>>> =
    LazyLock::new(|| RwLock::new(Box::new("".to_owned())));

pub async fn index_handler(State(app_state): State<AppState>) -> Result<Response<Body>, AppError> {
    if let Ok(index_html) = INDEX_HTML.read() {
        let html = index_html.deref().deref().to_owned();
        if !html.is_empty() {
            let body = Body::from(html);
            return Ok((StatusCode::OK, body).into_response());
        }
    }

    let html = shell(app_state.leptos_options.clone()).to_html();
    if let Ok(mut index_html) = INDEX_HTML.write() {
        **index_html = html.to_owned();
    }
    let body = Body::from(html);
    Ok((StatusCode::OK, body).into_response())
}
