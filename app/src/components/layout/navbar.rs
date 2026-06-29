use leptos::prelude::*;
use leptos_router::hooks::use_location;

use crate::components::ui::button_link::{ButtonLink, ButtonLinkColor, ButtonLinkWidth};

#[component]
pub fn Navbar() -> impl IntoView {
    let location = use_location();

    view! {
        <nav class="w-full relative bg-primary">

            <div class="px-1 py-1 sm:px-2 lg:px-4">
                <div class="flex justify-between h-9 md:h-14">
                    // Brand / Logo Area
                    <div class="shrink-0 flex items-center gap-4">
                        <a href="/" class="text-3xl md:text-4xl font-extrabold text-gray-800 pr-2 font-mono">TOOLS</a>
                        <ButtonLink label="Url encoder".to_owned() href="/urlEncoder".to_owned() button_width=ButtonLinkWidth::Auto
                            color=move || nav_button_color(location.pathname.get(), "/urlEncoder") />
                        <ButtonLink label="Json".to_owned() href="/json".to_owned() button_width=ButtonLinkWidth::Auto 
                            color=move || nav_button_color(location.pathname.get(), "/json") />
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
