use leptos::prelude::{ReadSignal, WriteSignal};

#[derive(Clone, Debug)]
pub struct RequestParams {
    pub url: ReadSignal<String>,
    pub set_url: WriteSignal<String>,
    pub method: ReadSignal<String>,
    pub set_method: WriteSignal<String>,
    pub body: ReadSignal<String>,
    pub set_body: WriteSignal<String>,
    pub content_type: ReadSignal<String>,
    pub set_content_type: WriteSignal<String>,
    pub accept: ReadSignal<String>,
    pub set_accept: WriteSignal<String>,
    pub user_agent: ReadSignal<String>,
    pub set_user_agent: WriteSignal<String>,
    pub accept_lang: ReadSignal<String>,
    pub set_accept_lang: WriteSignal<String>,
    pub custom_headers: ReadSignal<Vec<CustomHeader>>,
    pub set_custom_headers: WriteSignal<Vec<CustomHeader>>,
}

#[derive(Clone, Debug)]
pub struct CustomHeader {
    pub id: usize,
    pub name: ReadSignal<String>,
    pub set_name: WriteSignal<String>,
    pub value: ReadSignal<String>,
    pub set_value: WriteSignal<String>,
}
