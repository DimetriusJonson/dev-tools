use leptos::prelude::*;

#[derive(Clone)]
pub struct InnerEffect;

#[component]
pub fn CodeInner(code: String, lang: impl Fn() -> String + Send + Sync + 'static) -> impl IntoView {
    let lang_memo = Memo::new(move |_| lang());

    view! {

        {
            move || {
                let lang = lang_memo.get();

                if use_context::<InnerEffect>().is_none() {
                    let inner = {
                        let inner = crate::hljs::highlight(code.to_owned(), lang.to_owned());
                        inner
                    };
                    view! {
                        <pre class="whitespace-pre-wrap wrap-break-word break-all"><code inner_html=inner></code></pre>
                    }
                    .into_any()
                } else {
                    let (inner, set_inner) = signal(String::new());
                    {
                        let result = crate::hljs::highlight(code.to_owned(), lang.to_owned());
                        Effect::new(move |_| {
                            if let Some(r) = result.clone() {
                                set_inner.set(r)
                            }
                        });
                    };
                    view! {
                        <pre class="whitespace-pre-wrap wrap-break-word break-all"><code inner_html=inner></code></pre>
                    }
                    .into_any()
                }
        }}

    }
}
