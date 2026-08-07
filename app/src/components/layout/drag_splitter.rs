use leptos::{
    ev,
    html::Div,
    leptos_dom::{self},
    prelude::*,
};

use crate::common::{
    local_store::{delete_local_store_value, get_local_store_value, set_local_store_value}, ui_utils::{get_browser_height, get_browser_width},
};

#[component]
pub fn DragSplitter(
    target_ref: NodeRef<Div>,
    local_store_prop_name: impl Fn() -> String + Send + Sync + 'static,
    min_scr_ration: f64,
    max_scr_ration: f64,
    default_scr_ration: f64,
    #[prop(optional)] class_name: String,
    #[prop(optional)] allow_mobile: bool,
    #[prop(optional)] horizontal: bool,
) -> impl IntoView {
    let (dragging, set_dragging) = signal(false);
    let dragbar_ref = NodeRef::<Div>::new();
    let local_store_prop_name_memo = Memo::new(move |_| local_store_prop_name());

    let (screen_size, prop_name) = if horizontal { (get_browser_height().unwrap(), "height") } else { (get_browser_width().unwrap(), "width") };

    let (size, set_size) = signal(screen_size * default_scr_ration);

    let _ = Effect::new(move |_prev| {
        let init_size = get_local_store_value(
            &local_store_prop_name_memo.get(),
            (screen_size * default_scr_ration).to_string(),
        )
        .parse::<f64>()
        .unwrap();

        if (!is_mobile() || allow_mobile)
            && let Some(target_elem) = target_ref.get() {
                (*target_elem).style().set_property(prop_name, &format!("{}px", init_size)).unwrap();
            }
    });

    let _ = leptos_dom::helpers::window_event_listener(ev::resize, move |_ev| {
        if let Some(Some(target_elem)) = target_ref.try_get()
            && let Ok(screen_size) = if horizontal { get_browser_height() } else { get_browser_width()}
        {
            let default_size = screen_size * default_scr_ration;
            if !is_mobile() || allow_mobile {
                let current_size = size.get();

                let min_size = screen_size * min_scr_ration;
                let max_size = screen_size * max_scr_ration;

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
                && let Ok(screen_size) = if horizontal { get_browser_height() } else { get_browser_width()}
            {
                let min_size = screen_size * min_scr_ration;
                let max_size = screen_size * max_scr_ration;

                let rect = target_elem.get_bounding_client_rect();
                let new_size = if horizontal {ev.client_y() as f64 - rect.top()} else { ev.client_x() as f64 - rect.left()};

                if new_size > min_size && new_size < max_size {
                    (*target_elem)
                        .style()
                        .set_property(prop_name, &format!("{}px", new_size))
                        .unwrap();
                    set_size.set(new_size);
                    set_local_store_value(&local_store_prop_name_memo.get(), new_size.to_string());
                }
            }
    });

    let _ = leptos_dom::helpers::window_event_listener(ev::mouseup, move |_ev| {
        set_dragging.try_set(false);
    });

    let size_classes = if horizontal {"h-1 w-full"} else {"w-1 h-full"};

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
