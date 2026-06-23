use leptos::prelude::*;

#[component]
pub fn Navbar() -> impl IntoView {
    view! {
        <nav class="w-full relative bg-primary">

            <div class="px-1 py-1 sm:px-2 lg:px-4">
                <div class="flex justify-between h-9 md:h-14">
                    // Brand / Logo Area
                    <div class="shrink-0 flex items-center">
                        <a href="/" class="text-3xl md:text-4xl font-extrabold text-gray-800 pr-2 font-mono">TOOLS</a>
                        //<ButtonLink label="Пользователи".to_owned() href="/users".to_owned() color=Black button_width=Auto/>
                    </div>

                </div>
            </div>

        </nav>

    }
}
