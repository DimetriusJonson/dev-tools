use std::str::FromStr;

use crate::{
    common::local_store::set_local_store_value, components::ui::select_input::SelectInput, i18n::*,
};
use leptos::prelude::*;

#[component]
pub fn LanguageSelector() -> impl IntoView {
    let i18n = use_i18n();
    let locale = i18n.get_locale();
    let (value, set_value) = signal(locale.to_string());

    Effect::new(move |_| {
        #[cfg(not(feature = "ssr"))]
        {
            use crate::common::local_store::get_local_store_value;

            let browser_lang = crate::common::ui_utils::get_browser_language();
            let lang = get_local_store_value("lang", browser_lang);

            i18n.set_locale(match lang.as_str() {
                "ru" => Locale::ru,
                _ => Locale::en,
            });
            set_value.set(i18n.get_locale().to_string());
        }
    });

    view! {
        <SelectInput
            name="lang".to_owned()
            label=move || "lang".to_owned()
            not_selected_text=move || "".to_owned()
            options=move || Locale::get_all().into_iter().map(|l| (Some(l.to_string()), l.to_string())).collect()

            on_change=move |value: String| {
                let locale = Locale::from_str(&value.to_owned()).unwrap();
                i18n.set_locale(locale);
                set_local_store_value("lang", value);
            }
            value=value
            set_value=set_value
        />
    }
}
