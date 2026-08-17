use leptos::{
    ev,
    html::Div,
    leptos_dom::{self},
    prelude::*,
};
use web_sys::HtmlDivElement;

use crate::common::{
    local_store::{get_local_store_value, set_local_store_value},
    ui_utils::get_browser_width,
};

const MOBILE_WIDTH: f64 = 640.0;

#[component]
pub fn DragSplitter(
    target_ref: NodeRef<Div>,
    second_target_ref: NodeRef<Div>,
    local_store_prop_name: impl Fn() -> String + Send + Sync + 'static,
    min_ratio: f64,
    max_ratio: f64,
    default_ratio: f64,
    #[prop(optional)] class_name: String,
    #[prop(optional)] allow_mobile: bool,
    #[prop(optional)] horizontal: bool,
    #[prop(optional)] allow_hidden: bool,
) -> impl IntoView {
    let (dragging, set_dragging) = signal(false);
    let (mobile, set_mobile) = signal(get_browser_width().unwrap() < MOBILE_WIDTH);
    let dragbar_ref = NodeRef::<Div>::new();
    let local_store_prop_name_memo = Memo::new(move |_| local_store_prop_name());

    let prop_name = if horizontal { "height" } else { "width" };

    let (size, set_size) = signal(default_ratio);

    let set_element_size = |target_elem: HtmlDivElement,
                            second_target_elem: HtmlDivElement,
                            prop_name: &str,
                            new_size: f64| {
        //        console_log(&format!("{}={}%", local_store_prop_name_memo.get(), new_size));
        (*target_elem)
            .style()
            .set_property(prop_name, &format!("calc({}% - 0.5px)", new_size))
            .unwrap();

        (*second_target_elem)
            .style()
            .set_property(prop_name, &format!("calc({}% - 0.5px)", 100.0 - new_size))
            .unwrap();
    };

    let _ = leptos_dom::helpers::window_event_listener(ev::resize, move |_ev| {
        set_mobile.set(get_browser_width().unwrap() < MOBILE_WIDTH);
    });

    Effect::new(move || {
        let mut init_size =
            get_local_store_value(&local_store_prop_name_memo.get(), default_ratio.to_string())
                .parse::<f64>()
                .unwrap();

        if init_size == 0.0 {
            if let Some(target_elem) = target_ref.get() {
                target_elem.class_list().add_1("hidden").unwrap();
            }
        } else if init_size > max_ratio {
            init_size = default_ratio;
        }

        if (!mobile.get_untracked() || allow_mobile)
            && let Some(target_elem) = target_ref.get()
            && let Some(Some(second_target_elem)) = second_target_ref.try_get()
        {
            set_element_size(target_elem, second_target_elem, prop_name, init_size);
        }
    });

    let _ = leptos_dom::helpers::window_event_listener(ev::mousemove, move |ev| {
        if let Some(dragging) = dragging.try_get_untracked()
            && dragging
            && let Some(Some(target_elem)) = target_ref.try_get()
            && let Some(Some(second_target_elem)) = second_target_ref.try_get()
            && (!mobile.get_untracked() || allow_mobile)
        {
            let parent_rect = target_elem.parent_element().unwrap().get_bounding_client_rect();
            let target_rect = target_elem.get_bounding_client_rect();

            let (old_size, new_size) = if horizontal {
                (
                    target_rect.height(),
                    ((ev.client_y() as f64 - target_rect.top()) / parent_rect.height()) * 100.0,
                )
            } else {
                (
                    target_rect.width(),
                    ((ev.client_x() as f64 - target_rect.left()) / parent_rect.width()) * 100.0,
                )
            };

            if allow_hidden && old_size > 0.0 && new_size < min_ratio {
                if (old_size - new_size) > min_ratio / 2.0 {
                    if !target_elem.class_list().contains("hidden") {
                        target_elem.class_list().add_1("hidden").unwrap();
                    }
                    set_size.set(0.0);
                    set_element_size(
                        target_elem,
                        second_target_elem,
                        prop_name,
                        size.get_untracked(),
                    );
                    set_local_store_value(
                        &local_store_prop_name_memo.get(),
                        size.get_untracked().to_string(),
                    );
                }
            } else if new_size <= max_ratio && new_size >= min_ratio {
                if old_size == 0.0 && new_size > min_ratio / 2.0 {
                    if target_elem.class_list().contains("hidden") {
                        target_elem.class_list().remove_1("hidden").unwrap();
                    }

                    set_element_size(target_elem, second_target_elem, prop_name, new_size);

                    set_size.set(new_size);
                    set_local_store_value(&local_store_prop_name_memo.get(), new_size.to_string());
                    return;
                }

                set_element_size(target_elem, second_target_elem, prop_name, new_size);
                set_size.set(new_size);
                set_local_store_value(&local_store_prop_name_memo.get(), new_size.to_string());
            }
        }
    });

    let _ = leptos_dom::helpers::window_event_listener(ev::mouseup, move |_ev| {
        set_dragging.try_set(false);
    });

    let size_classes = if horizontal { "h-1 w-full" } else { "w-1 h-full" };

    view! {
        <div node_ref=dragbar_ref class={format!("hover:bg-blue-400/50 cursor-col-resize transition delay-300 duration-300 {} {}", size_classes, class_name)}
            class=(["bg-gray-700"], move || !dragging.get())
            class=(["bg-blue-400/50"], move || dragging.get())
            on:mousedown=move |e| {
                e.prevent_default();
                set_dragging.set(true);
            }
        />
    }
}
