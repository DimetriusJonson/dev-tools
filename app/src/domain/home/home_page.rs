use gloo_net::http::Request;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::components::ui::button::{Button, ButtonWidth};
use crate::components::ui::text_area::TextArea;

#[component]
pub fn HomePage() -> impl IntoView {
    let (xml, set_xml) = signal("".to_string());
    let (dst_xml, set_dst_xml) = signal("".to_string());
    let (in_progress, set_in_progress) = signal(false);

    let on_click = move |_| {
        let xml = xml.get();
        spawn_local(async move {
            set_in_progress.set(true);

            match Request::post("/format_xml").body(xml) {
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

    view! {
        <div class="flex-1 flex flex-row items-stretch gap-4 px-2 py-4 text-xs md:text-base">
            <TextArea
                name="xml".to_owned()
                class_name="resize-none".to_owned()
                placeholder="Вставьте xml".to_owned()
                value=xml
                set_value=set_xml
                rows="5".to_owned()
                on_change=|_| {}
            />

            <div class="flex items-center justify-center">
                <Button
                    label=">>".to_owned()
                    button_width=ButtonWidth::Auto
                    loading=in_progress
                    on_click=on_click
                    disabled=in_progress
                />
            </div>

            <TextArea
                name="dst_xml".to_owned()
                class_name="resize-none".to_owned()
                placeholder="Здесь будет отформатированный xml".to_owned()
                value=dst_xml
                set_value=set_dst_xml
                rows="5".to_owned()
                on_change=|_| {}
            />
        </div>
    }
}
