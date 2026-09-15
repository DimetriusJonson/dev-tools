use std::time::Duration;

use cookie::Cookie;
use leptos::prelude::*;
use leptos_router::{components::RoutingProgress, hooks::use_location};
use model::{
    constants::{RC_REQ_DATA_PARAM_NAME, RC_SRC_URL_PARAM_NAME},
    restclient::rest_client_request::RestClientRequest,
};
use url::Url;
use web_sys::HtmlIFrameElement;

use crate::{
    common::ui_utils::{create_cookie, get_browser_host_info},
    components::layout::message_banner::{Messages, show_error},
    domain::rest_client::{
        model::{request_params::RequestParams, rest_client_context::RestClientContext},
        util::html_previewer::{add_head_base_tag, clear_html_previewer, init_html_previewer},
    },
};

#[component]
pub fn RequestResultPreviewer(
    params: ReadSignal<RequestParams>,
    show_preview_html: RwSignal<bool>,
    proxy_allow: ReadSignal<bool>,
    body: ReadSignal<String>,
) -> impl IntoView {
    let messages = use_context::<Messages>().expect("Cant get messages context!");
    let rc_context = use_context::<RestClientContext>().expect("Failed get rc_context");
    let location = use_location();

    let (preview_sandbox, set_preview_sandbox) = signal("");
    let (preview_loading, set_preview_loading) = signal(false);

    Effect::watch(
        move || location.pathname.get(),
        move |_value, _prev, _| {
            clear_html_previewer();
        },
        false,
    );

    Effect::new(move || {
        clear_html_previewer();
    });

    Effect::watch(
        move || proxy_allow.get(),
        move |value, _prev, _| {
            if *value {
                set_preview_sandbox.set("allow-scripts allow-popups allow-same-origin");
            } else {
                set_preview_sandbox.set("allow-scripts allow-popups");
            }
        },
        false,
    );

    Effect::watch(
        move || show_preview_html.get(),
        move |value, _prev, _| {
            if *value {
                if let Err(err) = init_html_previewer(
                    proxy_allow.get_untracked(),
                    &rc_context.request.read_untracked().url,
                ) {
                    show_error(err, messages)
                }
            } else {
                clear_html_previewer();
            }
        },
        false,
    );

    Effect::watch(
        move || rc_context.request.get(),
        move |_value, _prev, _| {
            show_preview_html.set(false);
        },
        false,
    );

    let get_preview_src_doc = move || {
        set_preview_loading.set(true);
        let mut html = body.get();
        if proxy_allow.get_untracked() {
            None
        } else {
            add_head_base_tag(&mut html, &rc_context.request.read_untracked().url);
            Some(html)
        }
    };

    let get_preview_src = move || {
        set_preview_loading.set(true);
        if proxy_allow.get_untracked() {
            if let Ok(mut url) = Url::parse(&rc_context.request.get_untracked().url)
                && let Ok(host_info) = get_browser_host_info()
            {
                url.set_scheme(&host_info.0)
                    .unwrap_or_else(|_| panic!("Cant set url scheme {}", host_info.0));
                url.set_host(Some(&host_info.1))
                    .unwrap_or_else(|_| panic!("Cant set url host {} ", host_info.1));
                url.set_port(host_info.2)
                    .unwrap_or_else(|_| panic!("Cant set url port {:?}", host_info.2));
                url.query_pairs_mut()
                    .append_pair(RC_SRC_URL_PARAM_NAME, &rc_context.request.get_untracked().url);

                match params.read_untracked().get_body() {
                    Ok(body) => {
                        let request = RestClientRequest {
                            method: rc_context.request.get_untracked().method,
                            url: "".to_owned(),
                            headers: params
                                .read_untracked()
                                .headers
                                .read_untracked()
                                .iter()
                                .map(|h| (h.name.get_untracked(), h.value.get_untracked()))
                                .collect::<Vec<(String, String)>>(),
                            body,
                        };
                        if let Ok(json) =
                            serde_json::to_string(&request).map_err(|err| err.to_string())
                        {
                            url.query_pairs_mut().append_pair(RC_REQ_DATA_PARAM_NAME, &json);
                        }
                    }
                    Err(err) => {
                        show_error(err.to_string(), messages);
                        return None;
                    }
                };

                // Create cookies from current request
                let cookies: Vec<String> = params
                    .read_untracked()
                    .headers
                    .read_untracked()
                    .iter()
                    .filter(|h| h.name.read_untracked().to_lowercase() == "cookie")
                    .map(|h| h.value.get_untracked())
                    .collect();

                for cookie in cookies
                    .iter()
                    .flat_map(|value| value.split(';').into_iter())
                    .filter_map(|cookie| Cookie::parse_encoded(cookie.to_owned()).ok())
                {
                    create_cookie(cookie.name(), cookie.value(), None).unwrap();
                }

                Some(url.to_string())
            } else {
                None
            }
        } else {
            None
        }
    };

    view! {
        <Show when=move || { show_preview_html.get() }>
            <div class="flex-1 flex flex-col">
                <div class="progress-container pt-0 mt-0">
                    <RoutingProgress is_routing=preview_loading max_time=Duration::from_millis(250) />
                </div>
                <iframe class="flex-1 w-full"
                    src=get_preview_src
                    srcdoc=get_preview_src_doc
                    sandbox=preview_sandbox
                    on:load=move |event| {
                        let elem = event_target::<HtmlIFrameElement>(&event);
                        if let Some(cw) = elem.content_window() &&
                            let Ok(href) = cw.location().href() &&
                            let Ok(base_url) = url::Url::parse(&rc_context.request.read_untracked().url) &&
                            let Ok(mut href_url) = url::Url::parse(&href) &&
                            href_url.set_scheme(base_url.scheme()).is_ok() &&
                            href_url.set_host(base_url.host_str()).is_ok()
                            && let Err(err) = init_html_previewer(true, href_url.as_ref()) {
                                show_error(err, messages);
                            }
                        set_preview_loading.set(false);
                    }
                    on:error=move |_| {
                        set_preview_loading.set(false);
                    }
                >
                </iframe>
            </div>
        </Show>

    }
}
