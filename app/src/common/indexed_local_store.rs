use rexie::*;
use serde_json::Value;

pub async fn indexed_db_save_prop_value(
    store_name: &str,
    id: &str,
    prop_name: &str,
    prop_value: &str,
) -> Result<()> {
    let database = init_db().await?;

    save_prop_value(&database, store_name, id, prop_name, prop_value).await?;

    Ok(())
}

pub async fn get_prop_value_from_indexed_db(
    store_name: &str,
    id: &str,
    prop_name: &str,
) -> Result<Option<String>> {
    let database = init_db().await?;

    if let Some(value) = get_store_object(&database, store_name, id).await?
        && let Some(value) = value.as_object()
        && let Some(value) = value.get(prop_name)
        && let Some(value) = value.as_str()
    {
        return Ok(Some(value.to_owned()));
    }

    Ok(None)
}

pub async fn indexed_db_delete_key(store_name: &str, id: &str) -> Result<()> {
    let database = init_db().await?;

    Ok(delete_key(&database, store_name, id).await?)
}

async fn init_db() -> Result<Rexie> {
    let rexie = Rexie::builder("webdev_useful_tools_db")
        .version(1)
        .add_object_store(ObjectStore::new("xml"))
        .add_object_store(ObjectStore::new("json"))
        .add_object_store(ObjectStore::new("rc_request_response"))
        .build()
        .await?;

    Ok(rexie)
}

async fn save_prop_value(
    database: &Rexie,
    store_name: &str,
    id: &str,
    prop_name: &str,
    prop_value: &str,
) -> Result<Option<String>> {
    let transaction = database.transaction(&[store_name], TransactionMode::ReadWrite)?;

    let store = transaction.store(store_name)?;

    if let Some(data_js) = store.get(id.into()).await? {
        let data: Option<serde_json::Value> = serde_wasm_bindgen::from_value(data_js).unwrap();
        if let Some(mut data) = data {
            if let Some(data) = data.as_object_mut() {
                data.insert(prop_name.to_owned(), prop_value.into());

                let data_js = serde_wasm_bindgen::to_value(&data).unwrap();
                let id = store.put(&data_js, Some(&id.into())).await?;
                transaction.done().await?;

                return Ok(id.as_string());
            }
        }
    } else {
        let data = serde_json::json!({
            "id": 1,
            prop_name: prop_value,
        });

        let data_js = serde_wasm_bindgen::to_value(&data).unwrap();
        let id = store.add(&data_js, Some(&id.into())).await?;
        transaction.done().await?;

        return Ok(id.as_string());
    }

    return Ok(None);
}

async fn get_store_object(database: &Rexie, store_name: &str, id: &str) -> Result<Option<Value>> {
    let transaction = database.transaction(&[store_name], TransactionMode::ReadOnly)?;

    let store = transaction.store(store_name)?;
    if let Some(data_js) = store.get(id.into()).await? {
        let data: Option<serde_json::Value> = serde_wasm_bindgen::from_value(data_js).unwrap();
        return Ok(data);
    } else {
        return Ok(None);
    }
}

async fn delete_key(database: &Rexie, store_name: &str, id: &str) -> Result<()> {
    let transaction = database.transaction(&[store_name], TransactionMode::ReadWrite)?;

    let store = transaction.store(store_name)?;

    store.delete(id.into()).await
}
