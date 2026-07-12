use app::common::{app_error::AppError, app_state::ssr::AppState};
use axum::{
    body::to_bytes,
    extract::{RawQuery, Request, State},
    response::IntoResponse,
};
use http::{HeaderMap, HeaderValue, header};
use nanoid::nanoid;

use crate::{
    app_router::proxy_request_to_remote, common::{
        compress_utils::{compress_bytes, decompress_bytes}, dev_utils::{is_mime_image, parse_query_params}, image_utils::{convert_image_data_to_jpg, create_image_thumbnail},
    }, db::share_files_db::{
        create_share_file_in_db, delete_old_share_files_in_db, get_share_file_from_db,
        get_share_file_info_from_db, get_share_file_thumbnail_from_db,
    },
};

pub const DEFAULT_CONTENT_TYPE: &str = "application/octet-stream";
const MAX_FILE_SIZE: usize = 5 * 1024 * 1024;
pub const MIME_IMAGE_JPG: &str = "image/jpeg";

pub struct ShareFileUploadData {
    pub file_data: Vec<u8>,
    pub image_thumbnail: Option<Vec<u8>>,
    pub mime_type: String,
    pub external_id: String,
    pub file_name: String,
}

#[axum::debug_handler]
pub async fn share_file_upload(
    State(app_state): State<AppState>,
    RawQuery(query): RawQuery,
    headers: HeaderMap,
    request: Request,
) -> Result<impl IntoResponse, AppError> {
    let query_str = query.unwrap_or_default();
    let params = parse_query_params(&query_str);
    let file_name = params.get("file_name").unwrap_or(&"unknown_file");

    match app_state.pool {
        Some(pool) => {
            delete_old_share_files_in_db(&pool).await?;

            let prepared_data = share_file_prepare_for_upload(request, headers, file_name).await?;
            create_share_file_in_db(
                &prepared_data.external_id,
                file_name,
                &prepared_data.mime_type,
                prepared_data.file_data,
                prepared_data.image_thumbnail,
                &pool,
            )
            .await?;

            return Ok((prepared_data.external_id).into_response());
        }
        None => proxy_request_to_remote(app_state.remote_server_url.unwrap(), request).await,
    }
}

pub async fn share_file_prepare_for_upload(request: Request, headers: HeaderMap, file_name: &str) -> Result<ShareFileUploadData, AppError>  {
    let bytes = to_bytes(request.into_body(), MAX_FILE_SIZE)
        .await
        .map_err(AppError::system_error)?;
    let mut file_data = bytes.to_vec();
    let image_thumbnail;

    let default_content_type = HeaderValue::from_static(DEFAULT_CONTENT_TYPE);
    let mut content_type = headers.get("content-type").unwrap_or(&default_content_type).to_str().unwrap().to_owned();

    if is_mime_image(&content_type) {
        image_thumbnail = Some(
            create_image_thumbnail(&file_data, 300, 300).map_err(AppError::system_error)?,
        );
        if content_type != MIME_IMAGE_JPG {
            file_data =
                convert_image_data_to_jpg(&file_data).map_err(AppError::system_error)?;
            content_type = MIME_IMAGE_JPG.to_owned();
        }
    } else {
        image_thumbnail = None;
        file_data = compress_bytes(&file_data).map_err(AppError::system_error)?;
    }

    let external_id = nanoid!();

    Ok(ShareFileUploadData{ file_data, image_thumbnail, mime_type: content_type, external_id, file_name: file_name.to_owned() })

}

#[axum::debug_handler]
pub async fn share_file_download(
    State(app_state): State<AppState>,
    RawQuery(query): RawQuery,
    request: Request,
) -> Result<impl IntoResponse, AppError> {
    let query_str = query.unwrap_or_default();
    let params = parse_query_params(&query_str);
    let external_id = params.get("id").unwrap_or(&"");
    let thumbnail = params.get("thumbnail").unwrap_or(&"false").parse::<bool>().unwrap();

    match app_state.pool {
        Some(pool) => {
            if thumbnail {
                let mut headers = HeaderMap::new();
                headers.insert(header::CACHE_CONTROL, "public, max-age=3600".parse().unwrap());

                let image_thumbnail = get_share_file_thumbnail_from_db(external_id, &pool).await?;
                if let Some(image_thumbnail) = image_thumbnail {
                    headers.insert(header::CONTENT_TYPE, MIME_IMAGE_JPG.parse().unwrap());
                    Ok((headers, image_thumbnail).into_response())
                } else {
                    headers.insert(header::CONTENT_TYPE, DEFAULT_CONTENT_TYPE.parse().unwrap());
                    Ok((headers, vec![]).into_response())
                }
            } else {
                let share_file = get_share_file_from_db(external_id, &pool).await?;

                let mut mime_type = share_file.mime_type;
                if mime_type.is_empty() {
                    mime_type = DEFAULT_CONTENT_TYPE.to_owned();
                }

                let mut file_data = share_file.file_data;
                if !is_mime_image(&mime_type) {
                    file_data = decompress_bytes(file_data).map_err(AppError::system_error)?;
                }

                let mut headers = HeaderMap::new();
                headers.insert(header::CACHE_CONTROL, "public, max-age=3600".parse().unwrap());
                headers.insert(header::CONTENT_TYPE, mime_type.parse().unwrap());
                headers.insert(
                    header::CONTENT_DISPOSITION,
                    format!("attachment; filename=\"{}\"", share_file.file_name).parse().unwrap(),
                );

                Ok((headers, file_data).into_response())
            }
        }
        None => proxy_request_to_remote(app_state.remote_server_url.unwrap(), request).await,
    }
}

#[axum::debug_handler]
pub async fn share_file_info(
    State(app_state): State<AppState>,
    RawQuery(query): RawQuery,
    request: Request,
) -> Result<impl IntoResponse, AppError> {
    let query_str = query.unwrap_or_default();
    let params = parse_query_params(&query_str);
    let external_id = params.get("id").unwrap_or(&"");

    match app_state.pool {
        Some(pool) => {
            let share_file_info = get_share_file_info_from_db(external_id, &pool).await?;
            let is_image = is_mime_image(&share_file_info.mime_type);
            Ok(format!(
                "{}\n{}\n{}",
                share_file_info.file_name, share_file_info.mime_type, is_image
            )
            .into_response())
        }
        None => proxy_request_to_remote(app_state.remote_server_url.unwrap(), request).await,
    }
}
