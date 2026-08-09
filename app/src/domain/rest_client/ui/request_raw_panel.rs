use leptos::html::Div;
use leptos::{prelude::*, task::spawn_local};

use crate::common::ui_utils::copy_to_clipboard;
use crate::components::ui::button::{Button, ButtonHeight, ButtonWidth};
use crate::domain::rest_client::model::request_params::RequestParams;
use crate::domain::rest_client::util::curl_builder::{build_curl_bash_cmd, build_curl_win_cmd};
use crate::i18n::*;
use crate::components::layout::message_banner::{Messages, show_info};

#[component]
pub fn RequestRawPanel(
    request_raw: String,
    node_ref: NodeRef<Div>,
    params: ReadSignal<RequestParams>,
) -> impl IntoView {
    let messages = use_context::<Messages>().expect("Cant get messages context!");
    let i18n = use_i18n();

    let on_build_c_url = move |_| spawn_local(async move { 
        let cmd = build_curl_bash_cmd(&params.read_untracked());

        copy_to_clipboard(&cmd);
        show_info(t_string!(i18n, rest_client_curl_copied).to_owned(), messages);
    });

    let on_build_c_url_win = move |_| spawn_local(async move { 
        let cmd = build_curl_win_cmd(&params.read_untracked());

        copy_to_clipboard(&cmd);
        show_info(t_string!(i18n, rest_client_curl_copied).to_owned(), messages);
    });

    view! {
        <div class="flex flex-col text-xs md:text-base overflow-auto gap-4" node_ref=node_ref>
            <div class="flex gap-4">
                <Button
                    title=move || t_string!(i18n, rest_client_curl_build_btn_title).to_owned()
                    label=move || "cURL (Posix)".to_owned()
                    class_name="h-8".to_owned()
                    button_width=ButtonWidth::Auto
                    button_height=ButtonHeight::Custom
                    loading=move || false
                    on_click=on_build_c_url
                    disabled=move || false
                />
                <Button
                    title=move || t_string!(i18n, rest_client_curl_build_btn_title).to_owned()
                    label=move || "cURL (Windows)".to_owned()
                    class_name="h-8".to_owned()
                    button_width=ButtonWidth::Auto
                    button_height=ButtonHeight::Custom
                    loading=move || false
                    on_click=on_build_c_url_win
                    disabled=move || false
                />
            </div>
            <pre class="w-full whitespace-pre-wrap break-all">{request_raw}</pre>
        </div>
    }
}
