use leptos::{
    ev,
    html::Div,
    leptos_dom::{self},
    prelude::*,
};

use crate::common::{
    local_store::{delete_local_store_value, get_local_store_value, set_local_store_value},
    ui_utils::{get_browser_height, get_browser_width},
};

#[component]
pub fn DragSplitter(
    target_ref: NodeRef<Div>,
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
    let dragbar_ref = NodeRef::<Div>::new();
    let local_store_prop_name_memo = Memo::new(move |_| local_store_prop_name());

    let (screen_size, prop_name) = if horizontal {
        (get_browser_height().unwrap(), "height")
    } else {
        (get_browser_width().unwrap(), "width")
    };

    let (size, set_size) = signal(screen_size * default_ratio);

    let _ = Effect::new(move |_prev| {
        let init_size = get_local_store_value(
            &local_store_prop_name_memo.get(),
            (screen_size * default_ratio).to_string(),
        )
        .parse::<f64>()
        .unwrap();

        if init_size == 0.0 {
            if let Some(target_elem) = target_ref.get() {
                target_elem.class_list().add_1("hidden").unwrap();
            }
        }

        if (!is_mobile() || allow_mobile)
            && let Some(target_elem) = target_ref.get()
        {
            (*target_elem).style().set_property(prop_name, &format!("{}px", init_size)).unwrap();
        }
    });

    let _ = leptos_dom::helpers::window_event_listener(ev::resize, move |_ev| {
        if let Some(Some(target_elem)) = target_ref.try_get()
            && let Ok(screen_size) =
                if horizontal { get_browser_height() } else { get_browser_width() }
        {
            let default_size = screen_size * default_ratio;
            if !is_mobile() || allow_mobile {
                let current_size = size.get();

                let min_size = screen_size * min_ratio;
                let max_size = screen_size * max_ratio;

                if current_size < min_size {
                    (*target_elem)
                        .style()
                        .set_property(prop_name, &format!("{}px", min_size))
                        .unwrap();
                    set_size.set(min_size);
                    set_local_store_value(&local_store_prop_name_memo.get(), min_size.to_string());
                }
                if current_size > max_size {
                    (*target_elem)
                        .style()
                        .set_property(prop_name, &format!("{}px", max_size))
                        .unwrap();
                    set_size.set(max_size);
                    set_local_store_value(&local_store_prop_name_memo.get(), max_size.to_string());
                }
            } else {
                (*target_elem).style().remove_property(prop_name).unwrap();
                delete_local_store_value(&local_store_prop_name_memo.get());
                set_size.set(default_size);
            }
        }
    });

    let _ = leptos_dom::helpers::window_event_listener(ev::mousemove, move |ev| {
        if let Some(dragging) = dragging.try_get_untracked()
            && dragging
            && let Some(Some(target_elem)) = target_ref.try_get()
            && (!is_mobile() || allow_mobile)
            && let Ok(screen_size) =
                if horizontal { get_browser_height() } else { get_browser_width() }
        {
            let min_size = screen_size * min_ratio;
            let max_size = screen_size * max_ratio;

            let rect = target_elem.get_bounding_client_rect();
            let old_size = if horizontal { rect.height() } else { rect.width() };
            let new_size = if horizontal {
                ev.client_y() as f64 - rect.top()
            } else {
                ev.client_x() as f64 - rect.left()
            };

            if allow_hidden && old_size > 0.0 && new_size < min_size {
                if (old_size - new_size) > min_size / 2.0 {
                    if !target_elem.class_list().contains("hidden") {
                        target_elem.class_list().add_1("hidden").unwrap();
                    }
                    set_size.set(0.0);
                    (*target_elem)
                        .style()
                        .set_property(prop_name, &format!("{}px", size.get_untracked()))
                        .unwrap();
                    set_local_store_value(
                        &local_store_prop_name_memo.get(),
                        size.get_untracked().to_string(),
                    );
                }
            } else if new_size <= max_size && new_size >= min_size {
                if old_size == 0.0 && new_size > min_size / 2.0 {
                    if target_elem.class_list().contains("hidden") {
                        target_elem.class_list().remove_1("hidden").unwrap();
                    }
                    (*target_elem)
                        .style()
                        .set_property(prop_name, &format!("{}px", new_size))
                        .unwrap();
                    set_size.set(new_size);
                    set_local_store_value(&local_store_prop_name_memo.get(), new_size.to_string());
                    return;
                }

                (*target_elem).style().set_property(prop_name, &format!("{}px", new_size)).unwrap();
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
        <div node_ref=dragbar_ref class={format!("bg-gray-700 hover:bg-blue-400/50 cursor-col-resize transition-colors {} {}", size_classes, class_name)}
            on:mousedown=move |e| {
                e.prevent_default();
                set_dragging.set(true);
            }
        />
    }
}

fn is_mobile() -> bool {
    get_browser_width().unwrap() < 640.0
}
