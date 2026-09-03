use leptos::html::Div;
use leptos::prelude::*;
use web_sys::MouseEvent;

use crate::common::ui_utils::copy_to_clipboard;
use crate::components::layout::message_banner::{Messages, show_info};
use crate::components::ui::button::{Button, ButtonHeight, ButtonWidth};
use crate::domain::rest_client::model::request_info::RequestCommand;
use crate::domain::rest_client::model::request_params::{RequestParams};
use crate::domain::rest_client::model::rest_client_context::RestClientContext;
use crate::domain::rest_client::util::curl_builder::{build_curl_bash_cmd, build_curl_win_cmd};
use crate::i18n::*;

#[component]
pub fn RequestRawPanel(
    request_raw: ReadSignal<String>,
    node_ref: NodeRef<Div>,
    params: ReadSignal<RequestParams>,
) -> impl IntoView {
    let messages = use_context::<Messages>().expect("Cant get messages context!");
    let i18n = use_i18n();
    let rc_context = use_context::<RestClientContext>().expect("Failed get rc_context");

    let on_build_c_url = move |_| {
        let cmd = build_curl_bash_cmd(&params.read_untracked());

        copy_to_clipboard(&cmd);
        show_info(t_string!(i18n, rest_client_curl_copied).to_owned(), messages);
    };

    let on_build_c_url_win = move |_| {
        let cmd = build_curl_win_cmd(&params.read_untracked());

        copy_to_clipboard(&cmd);
        show_info(t_string!(i18n, rest_client_curl_copied).to_owned(), messages);
    };

    Effect::watch(
        move || rc_context.request.get(),
        move |value, _prev, _| {
            match value.command {
                RequestCommand::CopyCUrl => {
                    if let Ok(mouse_event) = MouseEvent::new("click") {
                        on_build_c_url(mouse_event) 
                    }
                },
                RequestCommand::CopyCUrlWin => {
                    if let Ok(mouse_event) = MouseEvent::new("click") {
                        on_build_c_url_win(mouse_event)
                    }
                }
                _ => (),
            }
            rc_context.request.write_untracked().command = RequestCommand::None;
        },
        false,
    );

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
            <pre class="h-0 w-full whitespace-pre-wrap break-all">{request_raw}</pre>
        </div>
    }
}
