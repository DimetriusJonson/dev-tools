use leptos::prelude::*;

#[derive(Clone)]
pub struct InnerEffect;

#[component]
pub fn CodeInner(code: String, lang: String) -> impl IntoView {
    // lang is currently unused for SSR, so just drop it now to use it to avoid warning.
    #[cfg(feature = "ssr")]
    drop(lang);
    if use_context::<InnerEffect>().is_none() {
        #[cfg(feature = "ssr")]
        let inner = Some(html_escape::encode_text(&code).into_owned());
        #[cfg(not(feature = "ssr"))]
        let inner = {
            let inner = crate::hljs::highlight(code, lang);
            leptos::logging::log!(
                "about to populate inner_html with: {inner:?}"
            );
            inner
        };
        view! {
            <pre><code inner_html=inner></code></pre>
        }
        .into_any()
    } else {
        let (inner, set_inner) = signal(String::new());
        #[cfg(feature = "ssr")]
        {
            set_inner.set(html_escape::encode_text(&code).into_owned());
        };
        #[cfg(not(feature = "ssr"))]
        {
            leptos::logging::log!("calling out to hljs::highlight");
            let result = crate::hljs::highlight(code, lang);
            Effect::new(move |_| {
                leptos::logging::log!(
                    "setting the result of hljs::highlight inside an effect"
                );
                if let Some(r) = result.clone() {
                    set_inner.set(r)
                }
            });
        };
        view! {
            <pre><code inner_html=inner></code></pre>
        }
        .into_any()
    }
}