use crate::common::constants::REMOTE_SERVER_HOST;
use crate::i18n::*;
use crate::{
    common::ui_utils::get_host_name, components::layout::language_selector::LanguageSelector,
};
use leptos::prelude::*;
use leptos_router::hooks::use_location;

use crate::components::ui::button_link::{ButtonLink, ButtonLinkColor, ButtonLinkWidth};

#[component]
pub fn Navbar() -> impl IntoView {
    let i18n = use_i18n();
    let location = use_location();

    view! {
    /*        <Await future=get_host_name() let:host_name>
                    <p>{host_name.to_owned()}</p>
            </Await>
     */
            <nav class="w-full relative bg-primary">
                <div class="px-1 py-2 sm:px-2 lg:px-4">
                    <div class="flex justify-between">
                        // Brand / Logo Area
                        <div class="flex flex-row flex-wrap gap-2 md:gap-4">
                            <ButtonLink label=move || "XML".to_owned() href="/".to_owned() button_width=ButtonLinkWidth::Auto
                                color=move || nav_button_color(location.pathname.get(), "/") />
                            <ButtonLink label=move || "URL".to_owned() href="/urlEncoder".to_owned() button_width=ButtonLinkWidth::Auto
                                color=move || nav_button_color(location.pathname.get(), "/urlEncoder") />
                            <ButtonLink label=move || "JSON".to_owned() href="/json".to_owned() button_width=ButtonLinkWidth::Auto
                                color=move || nav_button_color(location.pathname.get(), "/json") />
                            <ButtonLink label=move || t_display!(i18n, compare_btn_label).to_string() href="/compare_text".to_owned() button_width=ButtonLinkWidth::Auto
                                color=move || nav_button_color(location.pathname.get(), "/compare_text") />
                            <ButtonLink label=move || t_display!(i18n, share_file_btn_label).to_string() href="/share_file".to_owned() button_width=ButtonLinkWidth::Auto
                                color=move || nav_button_color(location.pathname.get(), "/share_file") />

                            <Transition>
                                {move || Suspend::new(async move {
                                    if get_host_name().await != REMOTE_SERVER_HOST {
                                        view! {
                                            <ButtonLink label=move || t_display!(i18n, rest_client_btn_label).to_string() href="/rest_client".to_owned() 
                                                button_width=ButtonLinkWidth::Auto
                                                color=move || nav_button_color(location.pathname.get(), "/rest_client") />
                                        }.into_any()
                                    } else {
                                        view! {}.into_any()
                                    }
                                })}
                            </Transition>
                        </div>

                        <div class="flex">
                            <LanguageSelector />
                        </div>

                    </div>
                </div>

            </nav>

        }
}

fn nav_button_color(curr_path: String, button_path: &str) -> ButtonLinkColor {
    if curr_path.as_str() == button_path { ButtonLinkColor::Black } else { ButtonLinkColor::Brown }
}
