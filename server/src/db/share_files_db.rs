use sqlx::{Pool, Postgres, Row};

use crate::{common::app_error::AppError, model::share_file::ShareFile};

pub async fn create_share_file_in_db(
    external_id: &str,
    file_name: &str,
    content_type: &str,
    file_data: Vec<u8>,
    image_thumbnail: Option<Vec<u8>>,
    pool: &Pool<Postgres>,
) -> Result<i64, AppError> {
    let row = sqlx::query(
        r#"
                INSERT INTO share_files (external_id, file_name, mime_type, file_data, image_thumbnail) 
                VALUES ($1, $2, $3, $4, $5)
                RETURNING id
            "#,
    )
    .bind(external_id)
    .bind(file_name)
    .bind(content_type)
    .bind(file_data)
    .bind(image_thumbnail)
    .fetch_one(pool)
    .await
    .map_err(AppError::system_error)?;

    Ok(row.get("id"))
}

pub async fn delete_old_share_files_in_db(pool: &Pool<Postgres>) -> Result<(), AppError> {
    sqlx::query("delete from share_files where created_at < now() - INTERVAL '3 day'")
        .execute(pool)
        .await
        .map_err(AppError::system_error)?;
    Ok(())
}

pub async fn get_share_file_thumbnail_from_db(
    external_id: &str,
    pool: &Pool<Postgres>,
) -> Result<Option<Vec<u8>>, AppError> {
    let row = sqlx::query("SELECT image_thumbnail FROM share_files WHERE external_id=$1")
        .bind(external_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::system_error)?;

    Ok(row.get("image_thumbnail"))
}

pub async fn get_share_file_from_db(
    external_id: &str,
    pool: &Pool<Postgres>,
) -> Result<ShareFile, AppError> {
    let row =
        sqlx::query("SELECT file_name, mime_type, file_data FROM share_files WHERE external_id=$1")
            .bind(external_id)
            .fetch_one(pool)
            .await
            .map_err(AppError::system_error)?;

    Ok(ShareFile {
        file_name: row.get("file_name"),
        file_data: row.get("file_data"),
        mime_type: row.get("mime_type"),
    })
}

pub async fn get_share_file_info_from_db(
    external_id: &str,
    pool: &Pool<Postgres>,
) -> Result<ShareFile, AppError> {
    let row =
        sqlx::query("SELECT file_name, mime_type FROM share_files WHERE external_id=$1")
            .bind(external_id)
            .fetch_one(pool)
            .await
            .map_err(AppError::system_error)?;

    Ok(ShareFile {
        file_name: row.get("file_name"),
        mime_type: row.get("mime_type"),
        file_data: vec![],
    })
}
