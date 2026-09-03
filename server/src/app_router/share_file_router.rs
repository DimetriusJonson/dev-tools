use app::model::share_file::{
    share_file_info_dto::ShareFileInfoDto, share_file_server::ShareFileServerDto,
};
use axum::{
    Json,
    body::to_bytes,
    extract::{RawQuery, Request, State},
    response::IntoResponse,
};
use http::{HeaderMap, HeaderValue, header};
use nanoid::nanoid;

use crate::{
    app_router::proxy_request_to_remote,
    common::{
        app_error::AppError,
        app_state::AppState,
        compress_utils::{compress_bytes, decompress_bytes},
        dev_utils::{is_mime_image, parse_query_params},
        image_utils::{convert_image_data_to_jpg, create_image_thumbnail},
        net_utils::get_local_addrs,
    },
    db::share_files_db::{
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
    let file_name = params.get("file_name").ok_or(AppError::system_error("parameter 'file_name' is empty".to_owned()))?;

    match app_state.pool {
        Some(pool) => {
            delete_old_share_files_in_db(&pool).await?;

            let prepared_data =
                share_file_prepare_for_upload(request, headers, file_name, MAX_FILE_SIZE).await?;
            create_share_file_in_db(
                &prepared_data.external_id,
                file_name,
                &prepared_data.mime_type,
                prepared_data.file_data,
                prepared_data.image_thumbnail,
                &pool,
            )
            .await?;

            Ok((prepared_data.external_id).into_response())
        }
        None => {
            if let Some(remote_server_url) = app_state.remote_server_url {
                proxy_request_to_remote(remote_server_url, request).await
            } else {
                Err(AppError::system_error("remote_server_url is empty"))
            }
        }
    }
}

pub async fn share_file_prepare_for_upload(
    request: Request,
    headers: HeaderMap,
    file_name: &str,
    max_file_size: usize,
) -> Result<ShareFileUploadData, AppError> {
    let bytes =
        to_bytes(request.into_body(), max_file_size).await.map_err(AppError::system_error)?;
    let mut file_data = bytes.to_vec();
    let image_thumbnail;

    let default_content_type = HeaderValue::from_static(DEFAULT_CONTENT_TYPE);
    let mut content_type = headers
        .get("content-type")
        .unwrap_or(&default_content_type)
        .to_str()
        .map_err(AppError::system_error)?
        .to_owned();

    if is_mime_image(&content_type) {
        image_thumbnail =
            Some(create_image_thumbnail(&file_data, 300, 300).map_err(AppError::system_error)?);
        if content_type != MIME_IMAGE_JPG {
            file_data = convert_image_data_to_jpg(&file_data).map_err(AppError::system_error)?;
            content_type = MIME_IMAGE_JPG.to_owned();
        }
    } else {
        image_thumbnail = None;
        file_data = compress_bytes(&file_data).map_err(AppError::system_error)?;
    }

    let external_id = nanoid!();

    Ok(ShareFileUploadData {
        file_data,
        image_thumbnail,
        mime_type: content_type,
        external_id,
        file_name: file_name.to_owned(),
    })
}

#[axum::debug_handler]
pub async fn share_file_download(
    State(app_state): State<AppState>,
    RawQuery(query): RawQuery,
    request: Request,
) -> Result<impl IntoResponse, AppError> {
    let query_str = query.unwrap_or_default();
    let params = parse_query_params(&query_str);
    let external_id = params.get("id").ok_or(AppError::system_error("parameter 'id' is empty".to_owned()))?;
    let thumbnail = params.get("thumbnail").map(|v| v.parse::<bool>().ok()).unwrap_or_default().unwrap_or_default();

    match app_state.pool {
        Some(pool) => {
            if thumbnail {
                let mut headers = HeaderMap::new();
                headers.insert(
                    header::CACHE_CONTROL,
                    "public, max-age=3600".parse().map_err(AppError::system_error)?,
                );

                let image_thumbnail = get_share_file_thumbnail_from_db(external_id, &pool).await?;
                if let Some(image_thumbnail) = image_thumbnail {
                    headers.insert(
                        header::CONTENT_TYPE,
                        MIME_IMAGE_JPG.parse().map_err(AppError::system_error)?,
                    );
                    Ok((headers, image_thumbnail).into_response())
                } else {
                    headers.insert(
                        header::CONTENT_TYPE,
                        DEFAULT_CONTENT_TYPE.parse().map_err(AppError::system_error)?,
                    );
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
                headers.insert(
                    header::CACHE_CONTROL,
                    "public, max-age=3600".parse().map_err(AppError::system_error)?,
                );
                headers.insert(
                    header::CONTENT_TYPE,
                    mime_type.parse().map_err(AppError::system_error)?,
                );
                headers.insert(
                    header::CONTENT_DISPOSITION,
                    format!("attachment; filename=\"{}\"", share_file.file_name)
                        .parse()
                        .map_err(AppError::system_error)?,
                );

                Ok((headers, file_data).into_response())
            }
        }
        None => {
            if let Some(remote_server_url) = app_state.remote_server_url {
                proxy_request_to_remote(remote_server_url, request).await
            } else {
                Err(AppError::system_error("remote_server_url empty"))
            }
        }
    }
}

#[axum::debug_handler]
pub async fn share_file_info(
    State(app_state): State<AppState>,
    RawQuery(query): RawQuery,
    request: Request,
) -> Result<impl IntoResponse, AppError> {
    match app_state.pool {
        Some(pool) => {
            let query_str = query.unwrap_or_default();
            let params = parse_query_params(&query_str);
            let external_id = params
                .get("id")
                .ok_or(AppError::system_error("parameter 'id' is empty".to_owned()))?;
            let share_file_info = get_share_file_info_from_db(external_id, &pool).await?;
            let is_image = is_mime_image(&share_file_info.mime_type);
            Ok(Json(ShareFileInfoDto {
                file_name: share_file_info.file_name,
                mime_type: share_file_info.mime_type,
                is_image,
            })
            .into_response())
        }
        None => {
            if let Some(remote_server_url) = app_state.remote_server_url {
                proxy_request_to_remote(remote_server_url, request).await
            } else {
                Err(AppError::system_error("remote_server_url empty"))
            }
        }
    }
}

#[axum::debug_handler]
pub async fn share_file_custom_servers_handler(
    State(app_state): State<AppState>,
) -> Result<Json<Vec<ShareFileServerDto>>, AppError> {
    if app_state.pool.is_some() {
        return Ok(Json(Vec::new()));
    }

    let addrs = get_local_addrs().map_err(AppError::system_error)?;
    let site_addr = app_state.leptos_options.site_addr;

    Ok(Json(
        addrs
            .iter()
            .map(|a| ShareFileServerDto {
                url: format!("http://{}:{}", a.0.to_owned(), site_addr.port()),
                description: format!("{} ({})", a.1, a.0),
            })
            .collect(),
    ))
}

pub async fn share_file_info_ex_handler(
    RawQuery(query): RawQuery,
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let query_str = query.unwrap_or_default();
    let params = parse_query_params(&query_str);
    let id =
        params.get("id").ok_or(AppError::system_error("parameter 'id' is empty".to_owned()))?;
    let local =
        params.get("local").map(|v| v.parse::<bool>().ok()).unwrap_or_default().unwrap_or_default();

    let site_addr = app_state.leptos_options.site_addr;

    let srv_name = if local { "share_local_file_info" } else { "share_file_info" };

    let response =
        reqwest::get(&format!("http://127.0.0.1:{}/{}?id={}", site_addr.port(), srv_name, id))
            .await
            .map_err(AppError::system_error)?;

    if response.status().is_success() {
        let share_file_info_dto =
            response.json::<ShareFileInfoDto>().await.map_err(AppError::system_error)?;
        Ok(Json(share_file_info_dto).into_response())
    } else {
        let response_text = response.text().await.map_err(AppError::system_error)?;
        Err(AppError::system_error(response_text))?
    }
}
