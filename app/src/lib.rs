#![recursion_limit = "512"]
pub mod app;
pub mod components;
pub mod domain;
pub mod common;
pub mod code_mirror;

include!(concat!(env!("OUT_DIR"), "/i18n/mod.rs"));
