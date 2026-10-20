use json_escape::unescape;
use leptos::prelude::*;
use crate::i18n::*;

use crate::common::json_processor::format_json;
use crate::domain::format_page::FormatPage;
use json_escape::escape_str;

#[component]
pub fn JsonPage() -> impl IntoView {
    let i18n = use_i18n();

    let format_func = move |value: String, ident| Ok(format_json(&value, ident));
    let escape_func = move |value: String| {
        let escaped_parts = escape_str(&value);
        let mut escaped_str = String::new();
        for part in escaped_parts {
            escaped_str.push_str(part);
        }
        Ok(escaped_str)
    };
    let unescape_func = move |value: String| {
        let res = unescape(&value).decode_utf8()?;
        Ok(res.to_string())
    };

    view! {

        <FormatPage
            name="json"
            lang="json"
            mime_type="application/json"
            format_file_service="/format_json"
            format_func
            escape_func
            unescape_func
            src_placeholder=move || t_string!(i18n, json_page_src_placeholder).to_owned()
        />

    }
}
