use std::time::Duration;

use leptos::html::{Div, Input};
use leptos::{ev, leptos_dom, prelude::*};
use web_sys::wasm_bindgen::JsCast;

use crate::components::layout::message_banner::{Messages, show_error};
use crate::components::ui::button::{
    Button, ButtonColor, ButtonHeight, ButtonTextSize, ButtonWidth,
};
use crate::components::ui::select_input::SelectInput;
use crate::components::ui::text_input::TextInput;
use crate::domain::rest_client::model::request_params::RestClientProject;
use crate::domain::rest_client::ui::request_popup_menu::RequestPopupMenu;
use crate::domain::rest_client::util::request_store::{
    get_stored_current_project, get_stored_projects, set_stored_current_project,
    set_stored_projects,
};
use crate::i18n::*;

#[component]
pub fn ProjectSelector(
    #[prop(optional)] class_name: String,
    project: ReadSignal<String>,
    set_project: WriteSignal<String>,
    #[prop(into)] on_delete: Callback<()>,
) -> impl IntoView {
    let i18n = use_i18n();
    let messages = use_context::<Messages>().expect("Cant get messages context!");

    let (old_project, set_old_project) = signal("".to_owned());
    let (projects, set_projects) = signal(Vec::new());
    let (popup_menu_show, set_popup_menu_show) = signal(false);
    let (edit_name_mode, set_edit_name_mode) = signal(false);
    let (edit_name, set_edit_name) = signal("".to_owned());
    let edit_name_ref = NodeRef::<Input>::new();
    let menu_ref = NodeRef::<Div>::new();

    let _ = Effect::new(move || {
        set_projects.set(get_stored_projects());
        set_project.set(get_stored_current_project());
    });

    Effect::watch(
        move || projects.get(),
        move |value, _prev, _| {
            set_stored_projects(value);
        },
        false,
    );

    Effect::watch(
        move || project.get(),
        move |value, _prev, _| set_stored_current_project(value.to_owned()),
        false,
    );

    let _ = leptos_dom::helpers::window_event_listener(ev::click, move |ev| {
        if let Some(popup_menu_show) = popup_menu_show.try_get()
            && popup_menu_show
        {
            if let Some(Some(target_element)) = menu_ref.try_get()
                && let Some(clicked_target) = ev.target()
            {
                let clicked_node: &web_sys::Node = clicked_target.unchecked_ref();
                if !target_element.contains(Some(clicked_node)) {
                    set_popup_menu_show.set(false);
                }
            }
            return;
        }

        if let Some(edit_name_mode) = edit_name_mode.try_get_untracked()
            && edit_name_mode
            && let Some(Some(target_element)) = edit_name_ref.try_get()
            && let Some(clicked_target) = ev.target()
        {
            let clicked_node: &web_sys::Node = clicked_target.unchecked_ref();
            if !target_element.contains(Some(clicked_node)) {
                set_edit_name_mode.set(false);
            }
        }
    });

    view! {
        <div class={format!("flex justify-center items-center {}", class_name)}>
            <Show when=move || edit_name_mode.get()
                fallback= move || view!{
                    <SelectInput
                        name="project".to_owned()
                        class_name="w-full".to_owned()
                        label=move || t_string!(i18n, rest_client_project_select_label).to_owned()
                        not_selected_text=move || t_string!(i18n, rest_client_project_not_selected).to_owned()
                        options=move || get_projects_options(projects.read().as_borrowed())
                        on_change=move |value| {
                            set_stored_current_project(value);
                        }
                        value=project
                        set_value=set_project
                    />

                    <div class="relative px-1 sm:px-2" node_ref=menu_ref>
                        <Button
                            label=move || "...".to_owned()
                            class_name="hover:bg-sky-500/80 w-8 h-5 pb-6".to_owned()
                            button_width=ButtonWidth::Custom
                            button_height=ButtonHeight::Custom
                            text_size=ButtonTextSize::Sm
                            color=ButtonColor::Custom
                            loading=move || false
                            disabled=move || false
                            on_click=move |_|{
                                set_popup_menu_show.set(true);
                            }
                        />

                        <Show when=move || popup_menu_show.get()>
                            <RequestPopupMenu class_name="absolute inset-0 z-50".to_owned()
                                items=move || {vec![
                                        ("create", t_string!(i18n, rest_client_explorer_create_project)),
                                        ("rename", t_string!(i18n, rest_client_explorer_rename_project)),
                                        ("delete", t_string!(i18n, rest_client_explorer_delete_project)),
                                        ]}
                                on_selected=move |val:(&'static str, &'static str)| {
                                        match val.0 {
                                            "create" => {
                                                set_edit_name_mode.set(true);
                                                set_edit_name.set("".to_owned());

                                                set_old_project.set(project.get());
                                                set_project.set("".to_owned());

                                                set_timeout(move || {
                                                    if let Some(input) = edit_name_ref.get() {
                                                        input.focus().unwrap();
                                                        input.select();
                                                        set_popup_menu_show.set(false);
                                                    }
                                                }, Duration::from_millis(250));
                                            },
                                            "rename" => {
                                                if let Ok(project_id) = project.get_untracked().parse::<i32>() {
                                                    if let Some(project_name) = projects.read_untracked().iter().filter(|p|p.id == project_id).map(|p|p.name.to_owned()).last() {
                                                        set_edit_name_mode.set(true);
                                                        set_edit_name.set(project_name);
                                                        set_old_project.set(project.get());
                                                        set_timeout(move || {
                                                            if let Some(input) = edit_name_ref.get() {
                                                                input.focus().unwrap();
                                                                input.select();
                                                                set_popup_menu_show.set(false);
                                                            }
                                                        }, Duration::from_millis(250));
                                                    }
                                                }
                                            },
                                            "delete" => {
                                                if let Ok(project_id) = project.get_untracked().parse::<i32>() {
                                                    on_delete.run(());

                                                    set_projects.write().retain(|p|p.id != project_id);
                                                    if let Some(prj) = projects.read_untracked().iter().nth(0) {
                                                        set_project.set(prj.id.to_string());
                                                    } else {
                                                        set_project.set("".to_owned());
                                                    }

                                                    set_popup_menu_show.set(false);
                                                }
                                            },
                                            _ => ()
                                        }
                                    }
                            />
                        </Show>
                    </div>
                }>
                    <TextInput
                        node_ref=edit_name_ref
                        name="request-name".to_owned()
                        class_name="w-full".to_owned()
                        placeholder=move || "Name".to_owned()
                        input_type="text".to_owned()
                        value=edit_name
                        set_value=set_edit_name
                        on_change=move |value: String| {
                            let val = value.trim().to_lowercase();
                            let project_id = project.get_untracked().parse::<i32>().unwrap_or(0);

                            if val.is_empty() {
                                show_error(t_string!(i18n, rest_client_empty_project_name).to_owned(), messages);
                                return;
                            }

                            if projects.read_untracked().iter().filter(|p|p.id != project_id)
                                .any(|p|p.name.to_lowercase() == val) {
                                    show_error(t_string!(i18n, rest_client_already_exist_project).to_owned(), messages);
                                return;
                            }

                            if project_id == 0 {
                                let project_id = generate_project_id();
                                let project = RestClientProject { id: project_id, name: value.trim().to_owned() };
                                set_projects.write().push(project);

                                set_project.set(project_id.to_string());
                            } else {
                                set_projects.write().iter_mut().filter(|p|p.id == project_id).for_each(|p| p.name = value.to_owned());
                            }
                            set_edit_name_mode.set(false);
                        }
                        on_cancel_change=move |_| {
                            let old_project_id = old_project.get_untracked();
                            set_project.set(old_project_id);
                            set_edit_name_mode.set(false);
                        }
                    />
            </Show>
        </div>
    }
}

fn get_projects_options(projects: &Vec<RestClientProject>) -> Vec<(Option<String>, String)> {
    projects.into_iter().map(|p| (Some(p.id.to_string()), p.name.to_owned())).collect()
}

fn generate_project_id() -> i32 {
    let projects = get_stored_projects();
    if !projects.is_empty()
        && let Some(id) = projects.iter().map(|p| p.id).max()
    {
        return id + 1;
    }

    1
}
