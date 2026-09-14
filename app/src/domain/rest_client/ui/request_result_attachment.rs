use crate::{
    common::ui_utils::save_file_to_disk, components::layout::message_banner::{Messages, show_error, show_info}, domain::rest_client::model::request_params::RequestParams, i18n::*,
};
use gloo_net::http::Request;
use leptos::{prelude::*, task::spawn_local};
use model::restclient::rest_client_request::RestClientRequest;

use crate::components::ui::button::{Button, ButtonWidth};

#[component]
pub fn RequestResultAttachment(
    params: ReadSignal<RequestParams>,
    attachment: ReadSignal<(String, String)>,
) -> impl IntoView {
    let i18n = use_i18n();
    let messages = use_context::<Messages>().expect("Cant get messages context!");

    let (in_progress, set_in_progress) = signal(false);

    let on_attachment_download_click = move |_| {
        spawn_local(async move {
            let mut headers = Vec::new();
            for header in params.read_untracked().headers.get_untracked().iter() {
                headers.push((header.name.get_untracked(), header.value.get_untracked()));
            }

            let attachment = attachment.get_untracked();
            let rc_request = RestClientRequest {
                method: "GET".to_owned(),
                url: attachment.0,
                headers,
                body: "".to_owned(),
            };

            set_in_progress.set(true);
            match Request::post("/rest_client_attachment_download").json(&rc_request) {
                Ok(request) => match request.send().await {
                    Ok(response) => match response.binary().await {
                        Ok(bytes) => {
                            let file_name = attachment.1;
                            match save_file_to_disk(bytes.to_vec(), &file_name, "application/json")
                            {
                                Ok(_) => show_info(
                                    t_display!(i18n, file_saved_file_msg, file_name).to_string(),
                                    messages,
                                ),
                                Err(err) => show_error(err, messages),
                            }
                        }
                        Err(err) => show_error(err.to_string(), messages),
                    },
                    Err(err) => show_error(err.to_string(), messages),
                },
                Err(err) => show_error(err.to_string(), messages),
            }

            set_in_progress.set(false);
        });
    };

    view! {
        <Show when=move || { !attachment.read().0.is_empty() }>
            <div class="flex-1 flex items-center justify-center">
                <Button
                    title=move || "".to_owned()
                    label=move || t_display!(i18n, rc_attachment_download_btn_label, file_name = attachment.get().1).to_string()
                    button_width=ButtonWidth::Auto
                    loading=move || in_progress.get()
                    on_click=on_attachment_download_click
                    disabled=move || in_progress.get()
                />
            </div>
        </Show>
    }
}
