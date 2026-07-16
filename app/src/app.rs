
use leptos::prelude::*;
use leptos_meta::{Meta, MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::components::{Outlet, ParentRoute, Route, Router, Routes};
use leptos_router::path;

use crate::components::layout::message_banner::MessageBanner;
use crate::components::layout::navbar::Navbar;
use crate::components::ui::button_link::{ButtonLink, ButtonLinkColor};
use crate::domain::compare_text::compare_text_page::CompareTextPage;
use crate::domain::json::json_page::JsonPage;
use crate::domain::share_file::share_file_upload_page::ShareFileUploadPage;
use crate::domain::share_file::share_file_view_page::ShareFileViewPage;
use crate::domain::url_encode::url_encode_page::UrlEncoderPage;
use crate::domain::xml::xml_page::XmlPage;
use crate::i18n::I18nContextProvider;

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
                <link rel="manifest" href="/manifest.json"/>
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
        <Title text="Useful tools for web developers"/>
        <Meta name="keywords" content="web, dev, tools, useful, xml, json, formatting, escape, share file, compare, text" />
        <Meta name="description" content="Useful tools for web developers" />

        // content for this welcome page
        <I18nContextProvider>
        <Router>
            <div class="flex flex-col h-dvh">
                <MessageBanner />

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
                                            color=move || ButtonLinkColor::Primary
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
                                <Route path=path!("") view=XmlPage />
                            </ParentRoute>
                            <Route path=path!("/urlEncoder") view=UrlEncoderPage />
                            <Route path=path!("/json") view=JsonPage />
                            <Route path=path!("/share_file") view=ShareFileUploadPage />
                            <Route path=path!("/share_file/view") view=ShareFileViewPage />
                            <Route path=path!("/compare_text") view=CompareTextPage />
                        </Routes>
                    </ErrorBoundary>

                </main>
            </div>
        </Router>
        </I18nContextProvider>
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