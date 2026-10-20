use crate::common::xml_processor::{escape_xml, format_xml};
use crate::domain::format_page::FormatPage;
use leptos::prelude::*;
use crate::i18n::*;

#[component]
pub fn XmlPage() -> impl IntoView {
    let i18n = use_i18n();

    let format_func = move |value: String, ident| Ok(format_xml(&value, ident)?);
    let escape_func = move |value: String| Ok(escape_xml(&value, true)?);
    let unescape_func = move |value: String| Ok(escape_xml(&value, false)?);

    view! {
        <FormatPage
            name="xml"
            lang="xml"
            mime_type="application/xml"
            format_file_service="/format_xml"
            format_func
            escape_func
            unescape_func
            src_placeholder=move || t_string!(i18n, xml_page_src_placeholder).to_owned()
        />
    }
}
