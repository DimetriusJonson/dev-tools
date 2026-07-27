use leptos::{
    ev,
    html::Div,
    leptos_dom::{self},
    prelude::*,
};

use crate::common::{
    local_store::{delete_local_store_value, get_local_store_value, set_local_store_value},
    ui_utils::get_browser_width,
};

#[component]
pub fn DragSplitter(
    target_ref: NodeRef<Div>,
    local_store_prop_name: &'static str,
    min_scr_ration: f64,
    max_scr_ration: f64,
    default_scr_ration: f64,
    #[prop(optional)] class_name: String,
    #[prop(optional)] allow_mobile: bool,
) -> impl IntoView {
    let (dragging, set_dragging) = signal(false);
    let dragbar_ref = NodeRef::<Div>::new();

    let screen_width = get_browser_width().unwrap() as f64;

    let (width, set_width) = signal(screen_width * default_scr_ration);

    let _ = Effect::new(move || {
        let init_width = get_local_store_value(
            local_store_prop_name,
            (screen_width * default_scr_ration).to_string(),
        )
        .parse::<f64>()
        .unwrap();

        if !is_mobile() || allow_mobile {
            if let Some(target_elem) = target_ref.get() {
                (*target_elem).style().set_property("width", &format!("{}px", init_width)).unwrap();
            }
        }
    });

    let _ = leptos_dom::helpers::window_event_listener(ev::resize, move |_ev| {
        if let Some(target_elem) = target_ref.get()
            && let Ok(screen_width) = get_browser_width()
        {
            let default_width = screen_width * default_scr_ration;
            if !is_mobile() || allow_mobile {
                let current_width = width.get();

                let min_width = screen_width * min_scr_ration;
                let max_width = screen_width * max_scr_ration;

                if current_width < min_width {
                    (*target_elem)
                        .style()
                        .set_property("width", &format!("{}px", min_width))
                        .unwrap();
                    set_width.set(min_width);
                    set_local_store_value(local_store_prop_name, min_width.to_string());
                }
                if current_width > max_width {
                    (*target_elem)
                        .style()
                        .set_property("width", &format!("{}px", max_width))
                        .unwrap();
                    set_width.set(max_width);
                    set_local_store_value(local_store_prop_name, max_width.to_string());
                }
            } else {
                (*target_elem).style().remove_property("width").unwrap();
                delete_local_store_value(local_store_prop_name);
                set_width.set(default_width);
            }
        }
    });

    let _ = leptos_dom::helpers::window_event_listener(ev::mousemove, move |ev| {
        if dragging.get_untracked()
            && let Some(target_elem) = target_ref.get()
            && (!is_mobile() || allow_mobile)
            && let Ok(screen_width) = get_browser_width()
        {
            let min_width = screen_width * min_scr_ration;
            let max_width = screen_width * max_scr_ration;

            let rect = target_elem.get_bounding_client_rect();
            let new_width = ev.client_x() as f64 - rect.left();

            if new_width > min_width && new_width < max_width {
                (*target_elem).style().set_property("width", &format!("{}px", new_width)).unwrap();
                set_width.set(new_width);
                set_local_store_value(local_store_prop_name, new_width.to_string());
            }
        }
    });

    let _ = leptos_dom::helpers::window_event_listener(ev::mouseup, move |_ev| {
        set_dragging.set(false);
    });

    view! {
        <div node_ref=dragbar_ref class={format!("w-1 bg-gray-700 hover:bg-blue-400/50 cursor-col-resize h-full transition-colors {}", class_name)}
            on:mousedown=move |e| {
                e.prevent_default();
                set_dragging.set(true);
            }
        />
    }
}

fn is_mobile() -> bool {
    get_browser_width().unwrap() < 768.0
}
