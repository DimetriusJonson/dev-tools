use gloo_net::http::Request;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::components::ui::button::{Button, ButtonWidth};
use crate::components::ui::select_input::SelectInput;
use crate::components::ui::text_area::TextArea;

#[cfg(not(feature = "ssr"))]
fn get_local_storage_value(key: &str, default: String) -> String {
    use gloo_storage::{LocalStorage, Storage};
    match LocalStorage::get(key) {
        Ok(value) => value,
        Err(_err) => default,
    }
}

#[cfg(feature = "ssr")]
fn get_local_storage_value(_key: &str, default: String) -> String {
    default
}

fn set_local_storage_value(key: &str, value: String) {
    use gloo_storage::{LocalStorage, Storage};
    LocalStorage::set(key, value).unwrap_or(());
}

#[component]
pub fn HomePage() -> impl IntoView {
    let (xml, set_xml) = signal(get_local_storage_value("src_xml", "".to_owned()));
    let (dst_xml, set_dst_xml) = signal("".to_owned());
    let (ident, set_ident) = signal(get_local_storage_value("xml_ident", "4".to_owned()));
    let (in_progress, set_in_progress) = signal(false);

    let on_format_click = move |_| {
        spawn_local(async move {
            set_in_progress.set(true);

            match Request::post("/format_xml")
                .query([("ident", ident.get_untracked())])
                .body(xml.get_untracked())
            {
                Ok(request) => match request.send().await {
                    Ok(response) => match response.text().await {
                        Ok(response_text) => set_dst_xml.set(response_text),
                        Err(_) => (),
                    },
                    Err(_) => (),
                },
                Err(_) => (),
            }
            set_in_progress.set(false);
        });
    };

    let on_copy_click = move |_| {
        if let Some(window) = web_sys::window() {
            let navigator = window.navigator();
            let clipboard = navigator.clipboard();
            let _ = clipboard.write_text(&dst_xml.get());
        }
    };

    view! {
        <div class="flex-1 flex flex-col md:flex-row gap-4 px-2 py-4 text-xs md:text-base">
            <TextArea
                name="xml".to_owned()
                class_name="flex-1 resize-none".to_owned()
                placeholder="Вставьте xml".to_owned()
                value=xml
                set_value=set_xml
                on_change=move |_| {
                    set_local_storage_value("src_xml", xml.get_untracked());
                }
            />

            <div class="flex flex-row md:flex-col gap-4 items-center justify-center">
                <SelectInput
                    name="ident".to_owned()
                    label="Отступ".to_owned()
                    options=move || {vec![(Some("2".to_owned()), "2 отступа".to_owned()), (Some("3".to_owned()), "3 отступа".to_owned()), (Some("4".to_owned()), "4 отступа".to_owned())]}
                    on_change=move |value| {
                        set_local_storage_value("xml_ident", value);
                    }
                    value=ident
                    set_value=set_ident
                />

                <Button 
                    label=">>".to_owned()
                    button_width=ButtonWidth::Md
                    loading=move || in_progress.get()
                    on_click=on_format_click
                    disabled=move || in_progress.get()
                />
            </div>

            <div class="flex-1 flex flex-col gap-4 w-full">
                <TextArea
                    name="dst_xml".to_owned()
                    class_name="flex-1 resize-none".to_owned()
                    placeholder="Здесь будет отформатированный xml".to_owned()
                    value=dst_xml
                    set_value=set_dst_xml
                    on_change=|_| {}
                />

                <Button 
                    label="Скопировать в буфер обмена".to_owned()
                    button_width=ButtonWidth::Auto
                    loading=move || in_progress.get()
                    on_click=on_copy_click
                    disabled=move || in_progress.get()
                />

            </div>
        </div>
    }
}
