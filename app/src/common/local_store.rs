
pub fn get_local_store_value(_key: &str, default: String) -> String {
    #[cfg(not(feature = "ssr"))]
    use gloo_storage::{LocalStorage, Storage};

    #[cfg(not(feature = "ssr"))]
    let val = LocalStorage::get(_key); 
    
    #[cfg(feature = "ssr")]
    let val: Result<String, std::convert::Infallible> = Ok(default.to_owned());

    match val {
        Ok(value) => value,
        Err(_err) => default,
    }
}

pub fn set_local_store_value(key: &str, value: String) {
    use gloo_storage::{LocalStorage, Storage};
    LocalStorage::set(key, value).unwrap_or(());
}
