use std::io::Cursor;

use axum::{
    extract::{RawQuery, State},
    response::IntoResponse,
};
use bytes::Bytes;
use http::{HeaderMap, HeaderValue, header};
use image::ImageFormat;
use nanoid::nanoid;
use sqlx::Row;

use crate::common::{app_error::AppError, app_state::AppState, dev_utils::parse_query_params};

const DEFAULT_CONTENT_TYPE: &str = "application/octet-stream";

pub async fn share_file_upload(
    State(app_state): State<AppState>,
    RawQuery(query): RawQuery,
    headers: HeaderMap,
    bytes: Bytes,
) -> Result<String, AppError> {
    let query_str = query.unwrap_or_default();
    let params = parse_query_params(&query_str);
    let file_name = params.get("file_name").unwrap_or(&"unknown_file");

    let default_content_type = HeaderValue::from_static(DEFAULT_CONTENT_TYPE);
    let content_type =
        headers.get("content-type").unwrap_or(&default_content_type).to_str().unwrap();

    match app_state.pool {
        Some(pool) => {
            let external_id = nanoid!();

            let file_data = bytes.to_vec();

            let image_thumbnail;
            if is_image(&content_type) {
                image_thumbnail = Some(
                    build_image_thumbnail(&file_data, 300, 300).map_err(AppError::system_error)?,
                );
            } else {
                image_thumbnail = None;
            }

            sqlx::query("INSERT INTO share_files (external_id, file_name, mime_type, file_data, image_thumbnail) VALUES ($1, $2, $3, $4, $5)")
                .bind(external_id.to_owned())
                .bind(file_name)
                .bind(content_type)
                .bind(file_data)
                .bind(image_thumbnail)
                .execute(&pool)
                .await
                .map_err(AppError::system_error)?;

            return Ok(external_id);
        }
        None => Err(AppError::system_error("Database unavailable!")),
    }
}

pub async fn share_file_download(
    State(app_state): State<AppState>,
    RawQuery(query): RawQuery,
) -> Result<impl IntoResponse, AppError> {
    let query_str = query.unwrap_or_default();
    let params = parse_query_params(&query_str);
    let external_id = params.get("id").unwrap_or(&"");
    let thumbnail = params.get("thumbnail").unwrap_or(&"false").parse::<bool>().unwrap();

    match app_state.pool {
        Some(pool) => {
            if thumbnail {
                let row =
                    sqlx::query("SELECT image_thumbnail FROM share_files WHERE external_id=$1")
                        .bind(external_id)
                        .fetch_one(&pool)
                        .await
                        .map_err(AppError::system_error)?;

                let mut headers = HeaderMap::new();
                headers.insert(header::CACHE_CONTROL, "public, max-age=3600".parse().unwrap());

                let image_thumbnail: Option<Vec<u8>> = row.get("image_thumbnail");
                if let Some(image_thumbnail) = image_thumbnail {
                    headers.insert(header::CONTENT_TYPE, "image/jpeg".parse().unwrap());
                    Ok((headers, image_thumbnail))
                } else {
                    headers.insert(header::CONTENT_TYPE, DEFAULT_CONTENT_TYPE.parse().unwrap());
                    Ok((headers, vec![]))
                }
            } else {
                let row = sqlx::query(
                    "SELECT file_name, mime_type, file_data FROM share_files WHERE external_id=$1",
                )
                .bind(external_id)
                .fetch_one(&pool)
                .await
                .map_err(AppError::system_error)?;

                let file_name: String = row.get("file_name");

                let mut mime_type: String = row.get("mime_type");
                if mime_type.is_empty() {
                    mime_type = DEFAULT_CONTENT_TYPE.to_owned();
                }

                let mut headers = HeaderMap::new();
                headers.insert(header::CACHE_CONTROL, "public, max-age=3600".parse().unwrap());
                headers.insert(header::CONTENT_TYPE, mime_type.parse().unwrap());
                headers.insert(
                    header::CONTENT_DISPOSITION,
                    format!("attachment; filename=\"{}\"", file_name).parse().unwrap(),
                );

                Ok((headers, row.get("file_data")))
            }
        }
        None => Err(AppError::system_error("Database unavailable!")),
    }
}

pub async fn share_file_info(
    State(app_state): State<AppState>,
    RawQuery(query): RawQuery,
) -> Result<String, AppError> {
    let query_str = query.unwrap_or_default();
    let params = parse_query_params(&query_str);
    let external_id = params.get("id").unwrap_or(&"");

    match app_state.pool {
        Some(pool) => {
            let row =
                sqlx::query("SELECT file_name, mime_type FROM share_files WHERE external_id=$1")
                    .bind(external_id)
                    .fetch_one(&pool)
                    .await
                    .map_err(AppError::system_error)?;

            let file_name: String = row.get("file_name");
            let mime_type: String = row.get("mime_type");
            let is_image = is_image(&mime_type);

            Ok(format!("{}\n{}\n{}", file_name, mime_type, is_image))
        }
        None => Err(AppError::system_error("Database unavailable!")),
    }
}

fn is_image(mime_type: &str) -> bool {
    match mime_type {
        "image/bmp" | "image/png" | "image/jpeg" | "image/webp" | "image/gif" | "image/apng" => {
            true
        }
        _ => false,
    }
}

fn build_image_thumbnail(
    src: &Vec<u8>,
    max_width: u32,
    max_height: u32,
) -> Result<Vec<u8>, image::ImageError> {
    let img = image::load_from_memory(src)?;

    let scaled = img.thumbnail(max_width, max_height);

    let mut dst = Vec::new();
    let mut cursor = Cursor::new(&mut dst);
    scaled.write_to(&mut cursor, ImageFormat::Jpeg)?;

    Ok(dst)
}
