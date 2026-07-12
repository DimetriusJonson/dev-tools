use std::sync::LazyLock;

use app::common::app_error::AppError;
use axum::{
    extract::{RawQuery, Request},
    response::IntoResponse,
};
use http::{HeaderMap, header};
use std::collections::HashMap;
use std::sync::Mutex;

use crate::{
    app_router::share_file_router::{DEFAULT_CONTENT_TYPE, MIME_IMAGE_JPG, ShareFileUploadData, share_file_prepare_for_upload}, common::{compress_utils::decompress_bytes, dev_utils::{is_mime_image, parse_query_params}},
};

static LOCAL_SHARE_DB: LazyLock<Mutex<HashMap<String, ShareFileUploadData>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[axum::debug_handler]
pub async fn share_local_file_upload(
    RawQuery(query): RawQuery,
    headers: HeaderMap,
    request: Request,
) -> Result<impl IntoResponse, AppError> {
    let query_str = query.unwrap_or_default();
    let params = parse_query_params(&query_str);
    let file_name = params.get("file_name").unwrap_or(&"unknown_file");

    let prepared_data = share_file_prepare_for_upload(request, headers, file_name).await?;

    let mut local_db = LOCAL_SHARE_DB.lock().unwrap();
    let external_id = prepared_data.external_id.to_owned();
    local_db.insert(external_id.to_owned(), prepared_data);

    return Ok((external_id.to_owned()).into_response());
}

#[axum::debug_handler]
pub async fn share_local_file_info(
    RawQuery(query): RawQuery,
) -> Result<impl IntoResponse, AppError> {
    let query_str = query.unwrap_or_default();
    let params = parse_query_params(&query_str);
    let external_id = params.get("id").unwrap_or(&"");

    let local_db = LOCAL_SHARE_DB.lock().unwrap();
    if let Some(data) = local_db.get(external_id.to_owned()) {
        let is_image = is_mime_image(&data.mime_type);
        Ok(format!("{}\n{}\n{}", data.file_name, data.mime_type, is_image).into_response())
    } else {
        Err(AppError::SystemError(format!("Not found file id={}!", external_id)))
    }
}

#[axum::debug_handler]
pub async fn share_local_file_download(
    RawQuery(query): RawQuery,
) -> Result<impl IntoResponse, AppError> {
    let query_str = query.unwrap_or_default();
    let params = parse_query_params(&query_str);
    let external_id = params.get("id").unwrap_or(&"");
    let thumbnail = params.get("thumbnail").unwrap_or(&"false").parse::<bool>().unwrap();

    let local_db = LOCAL_SHARE_DB.lock().unwrap();
    if let Some(data) = local_db.get(external_id.to_owned()) {
        if thumbnail {
            let mut headers = HeaderMap::new();
            headers.insert(header::CACHE_CONTROL, "public, max-age=3600".parse().unwrap());

            if let Some(image_thumbnail) = &data.image_thumbnail {
                headers.insert(header::CONTENT_TYPE, MIME_IMAGE_JPG.parse().unwrap());
                Ok((headers, image_thumbnail.clone()).into_response())
            } else {
                headers.insert(header::CONTENT_TYPE, DEFAULT_CONTENT_TYPE.parse().unwrap());
                Ok((headers, vec![]).into_response())
            }
        } else {
            let mut mime_type = data.mime_type.to_owned();
            if mime_type.is_empty() {
                mime_type = DEFAULT_CONTENT_TYPE.to_owned();
            }

            let mut file_data = data.file_data.clone();
            if !is_mime_image(&mime_type) {
                file_data = decompress_bytes(file_data).map_err(AppError::system_error)?;
            }

            let mut headers = HeaderMap::new();
            headers.insert(header::CACHE_CONTROL, "public, max-age=3600".parse().unwrap());
            headers.insert(header::CONTENT_TYPE, mime_type.parse().unwrap());
            headers.insert(
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}\"", data.file_name).parse().unwrap(),
            );

            Ok((headers, file_data).into_response())
        }
    } else {
        Err(AppError::SystemError(format!("Not found file id={}!", external_id)))
    }
}
