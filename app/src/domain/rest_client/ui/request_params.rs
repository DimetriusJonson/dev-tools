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
pub struct RequestInfo {
    pub id: i32,
    pub url: String,
    pub name: String,
    pub method: String,
    pub autorun: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CustomHeader {
    pub id: usize,
    pub name: ReadSignal<String>,
    pub set_name: WriteSignal<String>,
    pub value: ReadSignal<String>,
    pub set_value: WriteSignal<String>,
}

impl RequestInfo {
    pub fn new(id: i32, url: String, name: String, method: String) -> Self {
        Self { id, url, method, name, autorun: false }
    }

    pub fn clone_and_run(&self) -> Self {
        let mut info = self.clone();
        info.autorun = true;
        info
    }

    pub fn new_empty() -> Self {
        Self { id: 0, url: "".to_owned(), method: "".to_owned(), name: "".to_owned(), autorun: false  }
    }

    pub fn display_name(&self) -> String {
        if !self.name.is_empty() { self.name.to_owned() } else { self.url.to_owned() }
    }
}
