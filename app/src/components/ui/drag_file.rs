use leptos::prelude::*;
use web_sys::{DragEvent, File, HtmlDivElement};

pub type SelectOption = (Option<String>, String);

#[component]
pub fn DragFile(#[prop(into)] on_drop_file: Callback<File>) -> impl IntoView {
    view! {
        <div class="w-full max-w-md ">
            <div id="dropZone" class="w-full h-52 border-2 border-dashed border-green-500 rounded-xl flex flex-col justify-center items-center
                transition-all duration-300 ease-in-out cursor-pointer 
                bg-white dark:bg-dark-bg 
                hover:bg-green-50/30
                dark:hover:bg-dark-bg
                dark:hover:ring-primary
                dark:hover:border-primary

                text-green-500 
                dark:hover:text-primary
                "

                on:dragenter=active_drop_zone_handler
                on:dragover=active_drop_zone_handler
                on:dragleave=deactive_drop_zone_handler

                on:drop=move |event: DragEvent| {
                    deactive_drop_zone_handler(event.clone());

                    let dt = event.data_transfer().unwrap();
                    let files = dt.files().unwrap();

                    if files.length() > 0 {
                        on_drop_file.run(files.get(0).unwrap());
                    }
                }

            >
                <svg id="dragImage" class="w-12 h-12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
                    <polyline points="17 8 12 3 7 8"></polyline>
                    <line x1="12" y1="3" x2="12" y2="15"></line>
                </svg>
                <p id="dragText" class="mt-3 text-gray-600 text-base font-medium">Перетащите файл сюда</p>
            </div>
        </div>
    }
}

fn active_drop_zone_handler(event: DragEvent) {
    event.prevent_default();
    event.stop_propagation();

    if event.target() == event.current_target() {
        let drop_zone = event_target::<HtmlDivElement>(&event);
        drop_zone.class_list().add_1("scale-102").unwrap();
    }
}

fn deactive_drop_zone_handler(event: DragEvent) {
    event.prevent_default();
    event.stop_propagation();

    if event.target() == event.current_target() {
        let drop_zone = event_target::<HtmlDivElement>(&event);
        drop_zone.class_list().remove_1("scale-102").unwrap();
    }
}
