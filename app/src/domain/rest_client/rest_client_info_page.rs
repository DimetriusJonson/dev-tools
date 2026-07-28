use crate::i18n::*;
use leptos::prelude::*;

#[component]
pub fn RestClientInfoPage() -> impl IntoView {
    let i18n = use_i18n();

    view! {
        <div class="flex flex-col dark:text-white text-xs md:text-base justify-center items-center">
            <p class="font-medium pt-4">{t!(i18n, rest_client_info_1)}</p>
            <p>{t!(i18n, rest_client_info_2, <b> = <b />)}</p>
            <p>{t!(i18n, rest_client_info_3)}</p>
            <p>{t!(i18n, rest_client_info_4, <a> = <a class="text-blue-500 font-medium" href="https://github.com/DimetriusJonson/dev-tools/releases"/>)}</p>
        </div>
    }
}

