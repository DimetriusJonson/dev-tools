pub fn get_local_store_value(_key: &str, default: String) -> String {
    use gloo_storage::{LocalStorage, Storage};

    let val = LocalStorage::get(_key);

    match val {
        Ok(value) => value,
        Err(_err) => default,
    }
}

pub fn set_local_store_value(key: &str, value: String) {
    use gloo_storage::{LocalStorage, Storage};
    LocalStorage::set(key, value).unwrap_or(());
}

pub fn delete_local_store_value(key: &str) {
    use gloo_storage::{LocalStorage, Storage};
    LocalStorage::delete(key);
}
