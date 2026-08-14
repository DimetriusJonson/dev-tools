use leptos::prelude::*;

#[component]
pub fn CodeInner(code: String, lang: impl Fn() -> String + Send + Sync + 'static) -> impl IntoView {
    let lang_memo = Memo::new(move |_| lang());

    view! {
        {
            move || {
                let lang = lang_memo.get();

                // lang is currently unused for SSR, so just drop it now to use it to avoid warning.
                #[cfg(feature = "ssr")]
                drop(lang);

                #[cfg(feature = "ssr")]
                let inner = Some(html_escape::encode_text(&code).into_owned());

                #[cfg(not(feature = "ssr"))]
                let inner = crate::hljs::highlight(code.to_owned(), lang.to_owned());
                
                view! {
                    <pre class="whitespace-pre-wrap wrap-break-word break-all"><code inner_html=inner></code></pre>
                }
                .into_any()
        }}
    }
}
