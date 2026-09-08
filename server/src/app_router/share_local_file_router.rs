use std::sync::LazyLock;

use crate::common::app_error::AppError;
use axum::{Json, extract::Request, response::IntoResponse};
use http::{HeaderMap, header};
use model::share_file::share_file_info_dto::ShareFileInfoDto;
use std::collections::HashMap;
use std::sync::Mutex;

use crate::{
    app_router::share_file_router::{
        DEFAULT_CONTENT_TYPE, MIME_IMAGE_JPG, ShareFileUploadData, share_file_prepare_for_upload,
    },
    common::{
        compress_utils::decompress_bytes,
        dev_utils::{is_mime_image, extract_uri_query_params},
    },
};

static LOCAL_SHARE_DB: LazyLock<Mutex<HashMap<String, ShareFileUploadData>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[axum::debug_handler]
pub async fn share_local_file_upload(request: Request) -> Result<impl IntoResponse, AppError> {
    let uri = request.uri().clone();
    let params = extract_uri_query_params(&uri);
    let file_name = params.get("file_name").unwrap_or(&"unknown_file");

    let prepared_data = share_file_prepare_for_upload(request, file_name, usize::MAX).await?;

    let mut local_db = LOCAL_SHARE_DB.lock().map_err(AppError::system_error)?;
    let external_id = prepared_data.external_id.to_owned();
    local_db.insert(external_id.to_owned(), prepared_data);

    Ok((external_id.to_owned()).into_response())
}

#[axum::debug_handler]
pub async fn share_local_file_info(request: Request) -> Result<impl IntoResponse, AppError> {
    let params = extract_uri_query_params(request.uri());
    let external_id = params.get("id").unwrap_or(&"");

    let local_db = LOCAL_SHARE_DB.lock().map_err(AppError::system_error)?;
    if let Some(data) = local_db.get(external_id.to_owned()) {
        let is_image = is_mime_image(&data.mime_type);
        Ok(Json(ShareFileInfoDto {
            file_name: data.file_name.to_owned(),
            mime_type: data.mime_type.to_owned(),
            is_image,
        })
        .into_response())
    } else {
        Err(AppError::SystemError(format!("Not found file id={}!", external_id)))
    }
}

#[axum::debug_handler]
pub async fn share_local_file_download(request: Request) -> Result<impl IntoResponse, AppError> {
    let params = extract_uri_query_params(request.uri());
    let external_id = params.get("id").unwrap_or(&"");
    let thumbnail = params
        .get("thumbnail")
        .unwrap_or(&"false")
        .parse::<bool>()
        .map_err(AppError::system_error)?;

    let local_db = LOCAL_SHARE_DB.lock().map_err(AppError::system_error)?;
    if let Some(data) = local_db.get(external_id.to_owned()) {
        if thumbnail {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CACHE_CONTROL,
                "public, max-age=3600".parse().map_err(AppError::system_error)?,
            );

            if let Some(image_thumbnail) = &data.image_thumbnail {
                headers.insert(
                    header::CONTENT_TYPE,
                    MIME_IMAGE_JPG.parse().map_err(AppError::system_error)?,
                );
                Ok((headers, image_thumbnail.clone()).into_response())
            } else {
                headers.insert(
                    header::CONTENT_TYPE,
                    DEFAULT_CONTENT_TYPE.parse().map_err(AppError::system_error)?,
                );
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
            headers.insert(
                header::CACHE_CONTROL,
                "public, max-age=3600".parse().map_err(AppError::system_error)?,
            );
            headers
                .insert(header::CONTENT_TYPE, mime_type.parse().map_err(AppError::system_error)?);
            headers.insert(
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}\"", data.file_name)
                    .parse()
                    .map_err(AppError::system_error)?,
            );

            Ok((headers, file_data).into_response())
        }
    } else {
        Err(AppError::SystemError(format!("Not found file id={}!", external_id)))
    }
}
