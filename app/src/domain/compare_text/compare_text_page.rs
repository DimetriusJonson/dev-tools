use gloo_net::http::Request;
use leptos::task::spawn_local;
use leptos::prelude::*;
use web_sys::FormData;

use crate::common::local_store::{get_local_store_value, set_local_store_value};
use crate::components::layout::message_banner::{Messages, show_error};
use crate::components::ui::button::{Button, ButtonWidth};
use crate::components::ui::text_area::TextArea;

#[component]
pub fn CompareTextPage() -> impl IntoView {
    let messages = use_context::<Messages>().expect("Cant get messages context!");

    let (tab_selected, set_tab_selected) = signal(0);

    let (text1, set_text1) = signal(get_local_store_value("compare_text1", "".to_owned()));
    let (text2, set_text2) = signal(get_local_store_value("compare_text2", "".to_owned()));
    let (dst_left, set_dst_left) = signal("".to_owned());
    let (dst_right, set_dst_right) = signal("".to_owned());
    let (in_progress, set_in_progress) = signal(false);

    let on_compare_click = move |_| {
        spawn_local(async move {
            set_in_progress.set(true);

            let form_data = FormData::new().unwrap();
            form_data.append_with_str("text1", text1.read_untracked().as_str()).unwrap();
            form_data.append_with_str("text2", text2.read_untracked().as_str()).unwrap();

            match Request::post("/compare_text")
                .body(form_data)
            {
                Ok(request) => match request.send().await {
                    Ok(response) => match response.text().await {
                        Ok(response_text) => { 
                            let texts:Vec<&str> = response_text.split("\n$$$---$$$\n").collect();
                            set_dst_left.set(texts[0].into());
                            set_dst_right.set(texts[1].into());
                            set_tab_selected.set(1);
                        },
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

        <div class="flex-1 px-2 ">
            // Tab Headers
            <div class="flex border-b border-gray-200 text-sm font-medium text-center focus:outline-none" role="tablist">
                <button role="tab" aria-selected="true" 
                    class="flex-1 py-2.5 border-b-2 cursor-pointer"
                    class:border-blue-600=move || tab_selected.get() == 0
                    class:text-black=move || tab_selected.get() == 0
                    class:dark:text-white=move || tab_selected.get() == 0
                    class:text-gray-500=move || tab_selected.get() != 0
                    on:click=move |_event| {
                        set_tab_selected.set(0)
                    } 
                >
                Исходные данные
                </button>
                <button role="tab" aria-selected="false" 
                    class="flex-1 py-2.5 border-b-2 cursor-pointer"
                    class:border-blue-600=move || tab_selected.get() == 1
                    class:text-black=move || tab_selected.get() == 1
                    class:dark:text-white=move || tab_selected.get() == 1
                    class:text-gray-500=move || tab_selected.get() != 1
                    on:click=move |_event| {
                        set_tab_selected.set(1)
                    } 
                    >
                Результат
                </button>
            </div>

            //Tab Content Panels
            <div class="mt-4">
                <div class="flex flex-col md:flex-row gap-4 py-4 text-xs md:text-base min-h-0 overflow-y-auto h-[76dvh] md:h-[87dvh]"
                    class:block=move || tab_selected.get() == 0
                    class:hidden=move || tab_selected.get() != 0
                    >
                    <TextArea
                        name="text1".to_owned()
                        class_name="flex-1 resize-none".to_owned()
                        placeholder="Вставьте первый текст".to_owned()
                        value=text1
                        set_value=set_text1
                        on_change=move |_| {
                            set_local_store_value("compare_text1", text1.get_untracked());
                        }
                    />

                    <TextArea
                        name="text2".to_owned()
                        class_name="flex-1 resize-none".to_owned()
                        placeholder="Вставьте второй текст".to_owned()
                        value=text2
                        set_value=set_text2
                        on_change=move |_| {
                            set_local_store_value("compare_text2", text2.get_untracked());
                        }
                    />

                    <div class="flex flex-col gap-4 items-center justify-center">
                        <Button
                            label="Сравнить".to_owned()
                            button_width=ButtonWidth::Md
                            loading=move || in_progress.get()
                            on_click=on_compare_click
                            disabled=move || in_progress.get()
                        />
                        <Button
                            label="⇄".to_owned()
                            button_width=ButtonWidth::Md
                            loading=move || in_progress.get()
                            on_click=move |_| {
                                let temp_text = text1.get();
                                set_text1.set(text2.get());
                                set_text2.set(temp_text);

                                set_local_store_value("compare_text1", text1.get_untracked());
                                set_local_store_value("compare_text2", text2.get_untracked());
                            }
                            disabled=move || in_progress.get()
                        />
                    </div>
                </div>
                
                <div class="flex flex-col md:flex-row gap-4 py-4 text-xs md:text-base min-h-0 overflow-y-auto h-[76dvh] md:h-[87dvh]"
                    class:block=move || tab_selected.get() == 1
                    class:hidden=move || tab_selected.get() != 1
                >
                    <div class="flex-1 dark:text-white px-4" inner_html=move || dst_left />
                    <div class="flex-1 dark:text-white" inner_html=move || dst_right />
                </div>
            </div>
        </div>
    }
}
