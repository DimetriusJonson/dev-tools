#![recursion_limit = "256"]
pub mod app;
pub mod common;
pub mod components;
pub mod domain;
pub mod hljs;

include!(concat!(env!("OUT_DIR"), "/i18n/mod.rs"));
