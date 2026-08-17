use crate::common::text_comparator::compare_text;
use crate::components::layout::drag_splitter::DragSplitter;
use crate::components::layout::tabs::{TabItem, Tabs};
use crate::i18n::*;
use leptos::html::Div;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::common::local_store::{get_local_store_value, set_local_store_value};
use crate::components::ui::button::{Button, ButtonWidth};
use crate::components::ui::text_area::TextArea;

#[component]
pub fn CompareTextPage() -> impl IntoView {
    let i18n = use_i18n();

    let (tab_selected, set_tab_selected) = signal(0);
    let tab_source_ref = NodeRef::<Div>::new();
    let tab_result_ref = NodeRef::<Div>::new();

    let (text1, set_text1) = signal(get_local_store_value("compare_text1", "".to_owned()));
    let (text2, set_text2) = signal(get_local_store_value("compare_text2", "".to_owned()));
    let (dst_left, set_dst_left) = signal("".to_owned());
    let (dst_right, set_dst_right) = signal("".to_owned());
    let (in_progress, set_in_progress) = signal(false);

    let text1_ref = NodeRef::<Div>::new();
    let right_panel_ref = NodeRef::<Div>::new();

    let on_compare_click = move |_| {
        spawn_local(async move {
            set_in_progress.set(true);

            let text1_str = text1.read_untracked();
            let text2_str = text2.read_untracked();
            let texts = compare_text(text1_str.as_str(), text2_str.as_str());
            set_dst_left.set(texts.0);
            set_dst_right.set(texts.1);
            set_tab_selected.set(1);

            set_in_progress.set(false);
        });
    };

    view! {

        <div class="flex-1 px-2 pt-4">
            <Tabs tab_selected set_tab_selected items=move || vec![
                    TabItem::new_simple(t_string!(i18n, compare_page_source_tab), tab_source_ref),
                    TabItem::new_simple(t_string!(i18n, compare_page_result_tab), tab_result_ref)
                ] />

            //Tab Content Panels
            <div class="mt-4">
                <div node_ref=tab_source_ref class="flex flex-col md:flex-row gap-4 py-4 text-xs md:text-base min-h-0 overflow-y-auto h-[76dvh] md:h-[87dvh]">
                    <div class="flex-1 flex gap-x-4">
                        <div node_ref=text1_ref class="flex-1 sm:flex-none flex">
                            <TextArea
                                name="text1".to_owned()
                                class_name="w-full resize-none".to_owned()
                                placeholder=move || t_display!(i18n, compare_page_text1_placeholder).to_string()
                                value=text1
                                set_value=set_text1
                                on_change=move |_| {
                                    set_local_store_value("compare_text1", text1.get_untracked());
                                }
                            />
                        </div>

                        <DragSplitter
                            target_ref=text1_ref
                            second_target_ref=right_panel_ref
                            class_name="hidden md:block".to_owned()
                            local_store_prop_name=move || "compare_text1_width".to_owned()
                            min_ratio={20.0}
                            max_ratio={80.0}
                            default_ratio={50.0}/>

                        <div class="flex gap-4" node_ref=right_panel_ref>
                            <TextArea
                                name="text2".to_owned()
                                class_name="flex-1 resize-none".to_owned()
                                placeholder=move || t_display!(i18n, compare_page_text2_placeholder).to_string()
                                value=text2
                                set_value=set_text2
                                on_change=move |_| {
                                    set_local_store_value("compare_text2", text2.get_untracked());
                                }
                            />
                        </div>
                    </div>

                    <div class="flex flex-col gap-4 items-center justify-center">
                        <Button
                            title=move || "".to_owned()
                            label=move || t_display!(i18n, compare_btn_label).to_string()
                            button_width=ButtonWidth::Md
                            loading=move || in_progress.get()
                            on_click=on_compare_click
                            disabled=move || in_progress.get()
                        />
                        <Button
                            title=move || "".to_owned()
                            label=move || "⇄".to_owned()
                            button_width=ButtonWidth::Md
                            loading=move || false
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

                <div node_ref=tab_result_ref class="flex flex-col md:flex-row gap-4 py-4 text-xs md:text-base min-h-0 overflow-y-auto h-[76dvh] md:h-[87dvh]">
                    <div class="flex-1 dark:text-white overflow-x-auto w-full" inner_html=move || dst_left />
                    <div class="flex-1 dark:text-white overflow-x-auto w-full" inner_html=move || dst_right />
                </div>
            </div>
        </div>
    }
}
