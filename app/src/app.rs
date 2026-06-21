
use leptos::prelude::*;
use leptos_meta::{Meta, MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::components::{Outlet, ParentRoute, Route, Router, Routes};
use leptos_router::path;

use crate::components::layout::navbar::Navbar;
use crate::components::ui::button_link::{ButtonLink, ButtonLinkColor};
use crate::domain::home::home_page::HomePage;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/dev_tools.css"/>

        // sets the document title
        <Title text="Dev tools"/>
        <Meta name="keywords" content="dev, tools, xml, formatting, development" />
        <Meta name="description" content="Утилиты для разработчика. Форматирование XML." />

        // content for this welcome page
        <Router>
            <div class="flex flex-col h-dvh">
                <Navbar />
                <main class="flex flex-col flex-1 overflow-auto bg-white dark:bg-dark-bg">
                    //<MessageBanner />

                    <ErrorBoundary fallback=move |errors| {
                        let errors_clear = errors.clone();
                        let on_click = move |_| {
                            errors_clear.set(Errors::default());
                        };

                        view! {
                            <section class="container mx-auto pt-8">
                                <div class="bg-neutral-100 dark:bg-gray-950 p-8 rounded-lg shadow-md shadow-danger block text-center">
                                    <div class="text-5xl font-extrabold text-danger">500</div>
                                    <ul class="text-3xl text-gray-400">
                                        {move || errors.get()
                                            .into_iter()
                                            .map(|(_, error)| view! { <li>{format_error(error)}</li> })
                                            .collect::<Vec<_>>()
                                        }
                                    </ul>
                                    <div class="m-5">
                                        <ButtonLink
                                            color=ButtonLinkColor::Primary
                                            href="/".to_owned()
                                            label="Вернутся Домой".to_owned()
                                            on:click=on_click
                                        />
                                    </div>
                                </div>
                            </section>
                        }
                    }>

                        <Routes transition=true fallback=NotFound>
                            <ParentRoute path=path!("/") view=Outlet>
                                <Route path=path!("") view=HomePage />
                            </ParentRoute>
                        </Routes>
                    </ErrorBoundary>

                </main>
            </div>
        </Router>
    }
}

#[component]
pub fn NotFound() -> impl IntoView {
    view! {
        <section class="container mx-auto pt-8">
            <div class="bg-neutral-100 dark:bg-gray-950 p-8 rounded-lg shadow-md shadow-danger block text-center">
                <div class="text-5xl font-extrabold text-danger">404</div>
                <ul class="text-3xl text-gray-400">
                    <li>Страница не найдена</li>                
                </ul>
            </div>
        </section>
    }
}

fn format_error(error: Error) -> String {
    let msg = error.to_string();

    if let Some(pos) = msg.find('|') { msg[pos + 1..].to_string() } else { error.to_string() }
}