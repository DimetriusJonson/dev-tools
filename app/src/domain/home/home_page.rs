use gloo_net::http::Request;
use leptos::{prelude::*, task::spawn_local};

#[component]
pub fn HomePage() -> impl IntoView {
    let (xml, set_xml) = signal("".to_string());
    let (dst_xml, set_dst_xml) = signal("".to_string());

    let on_click = move |_| {
        let xml = xml.get();
        spawn_local(async move {
            let response = Request::post("/format_xml").body(xml).unwrap().send().await.unwrap();
            let response_text = response.text().await.unwrap();
            set_dst_xml.set(response_text);
        });

    };
    
    view! {
        <div class="flex h-screen w-full gap-4 p-4 text-xs md:text-base text-black">
            <textarea name="xml" class="flex-1 h-full resize-none rounded border p-2 focus:outline-none border-black" rows="auto" placeholder="Вставьте xml" bind:value=(xml, set_xml)></textarea>
            <button class="p-4 border" on:click=on_click>{">>"}</button>
            <textarea class="flex-1 h-full resize-none rounded border p-2 focus:outline-none border-black" rows="auto" placeholder="Здесь будет отформатированный xml" prop:value={dst_xml}></textarea>
        </div>
    }
}
