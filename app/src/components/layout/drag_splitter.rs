use leptos::{ev, html::Div, leptos_dom, prelude::*};

use crate::common::local_store::{get_local_store_value, set_local_store_value};

#[component]
pub fn DragSplitter(
    target_ref: NodeRef<Div>,
    local_store_prop_name: &'static str,
    min_width: i32,
    max_width: i32,
    default_width: i32,
) -> impl IntoView {
    let (dragging, set_dragging) = signal(false);
    let dragbar_ref = NodeRef::<Div>::new();

    let _ = Effect::new(move || {
        let init_width = get_local_store_value(local_store_prop_name, default_width.to_string())
            .parse::<i32>()
            .unwrap();
        if let Some(target_elem) = target_ref.get() {
            (*target_elem).style().set_property("width", &format!("{}px", init_width)).unwrap();
        }
    });

    let _ = leptos_dom::helpers::window_event_listener(ev::mousemove, move |ev| {
        if dragging.get()
            && let Some(target_elem) = target_ref.get()
        {
            let rect = target_elem.get_bounding_client_rect();
            let new_width = ev.client_x() - rect.left() as i32;

            if new_width > min_width && new_width < max_width {
                (*target_elem).style().set_property("width", &format!("{}px", new_width)).unwrap();

                set_local_store_value(local_store_prop_name, new_width.to_string());
            }
        }
    });

    let _ = leptos_dom::helpers::window_event_listener(ev::mouseup, move |_ev| {
        set_dragging.set(false);
    });

    view! {
        <div node_ref=dragbar_ref class="w-1 bg-gray-700 hover:bg-blue-400/50 cursor-col-resize h-full transition-colors"
            on:mousedown=move |e| {
                e.prevent_default();
                set_dragging.set(true);
            }
        />
    }
}
