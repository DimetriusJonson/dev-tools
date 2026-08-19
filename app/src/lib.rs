#![recursion_limit = "512"]
pub mod app;
pub mod components;
pub mod domain;
pub mod common;
pub mod model;
pub mod code_mirror;

include!(concat!(env!("OUT_DIR"), "/i18n/mod.rs"));

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
