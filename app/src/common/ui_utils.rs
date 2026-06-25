pub fn copy_to_clipboard(_data: &str) {
    #[cfg(not(feature = "ssr"))]
    if let Some(window) = web_sys::window() {
        let navigator = window.navigator();
        let clipboard = navigator.clipboard();
        let _ = clipboard.write_text(_data);
    }
}
