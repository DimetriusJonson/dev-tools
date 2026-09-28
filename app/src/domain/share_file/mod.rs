use leptos::prelude::{RwSignal, Set};

pub mod share_file_upload_page;
pub mod share_file_view_page;

#[derive(Clone, Default)]
pub struct ShareStorage(pub RwSignal<String>);

impl ShareStorage {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set_text(&self, text: &str) {
        self.0.set(text.to_owned());
    }
}
