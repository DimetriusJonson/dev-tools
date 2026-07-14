use leptos::prelude::*;
use leptos_router::hooks::use_location;

use crate::components::ui::button_link::{ButtonLink, ButtonLinkColor, ButtonLinkWidth};

#[component]
pub fn Navbar() -> impl IntoView {
    let location = use_location();

    view! {
        <nav class="w-full relative bg-primary">

            <div class="px-1 py-2 sm:px-2 lg:px-4">
                <div class="flex">
                    // Brand / Logo Area
                    <div class="flex flex-row flex-wrap gap-4">
                        <ButtonLink label="XML".to_owned() href="/".to_owned() button_width=ButtonLinkWidth::Auto
                            color=move || nav_button_color(location.pathname.get(), "/") />
                        <ButtonLink label="URL".to_owned() href="/urlEncoder".to_owned() button_width=ButtonLinkWidth::Auto
                            color=move || nav_button_color(location.pathname.get(), "/urlEncoder") />
                        <ButtonLink label="JSON".to_owned() href="/json".to_owned() button_width=ButtonLinkWidth::Auto 
                            color=move || nav_button_color(location.pathname.get(), "/json") />
                        <ButtonLink label="Сравнить".to_owned() href="/compare_text".to_owned() button_width=ButtonLinkWidth::Auto 
                            color=move || nav_button_color(location.pathname.get(), "/compare_text") />
                        <ButtonLink label="Поделится файлом".to_owned() href="/share_file".to_owned() button_width=ButtonLinkWidth::Auto 
                            color=move || nav_button_color(location.pathname.get(), "/share_file") />
                    </div>

                </div>
            </div>

        </nav>

    }
}

fn nav_button_color(curr_path: String, button_path: &str) -> ButtonLinkColor {
    if curr_path.as_str() == button_path {
        ButtonLinkColor::Black
    } else {
        ButtonLinkColor::Brown
    }
}
