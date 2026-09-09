use crate::db::DbPool;
use sqlx::prelude::FromRow;

#[derive(FromRow)]
pub struct ShareFile {
    pub file_name: String,
    pub file_data: Vec<u8>,
    pub mime_type: String,
}

pub async fn create_share_file_in_db(
    external_id: &str,
    file_name: &str,
    content_type: &str,
    file_data: Vec<u8>,
    image_thumbnail: Option<Vec<u8>>,
    pool: &DbPool,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
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
}

pub async fn delete_old_share_files_in_db(pool: &DbPool) -> Result<(), sqlx::Error> {
    sqlx::query("delete from share_files where created_at < now() - INTERVAL '3 day'")
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_share_file_thumbnail_from_db(
    external_id: &str,
    pool: &DbPool,
) -> Result<Option<Vec<u8>>, sqlx::Error> {
    sqlx::query_scalar("SELECT image_thumbnail FROM share_files WHERE external_id=$1")
        .bind(external_id)
        .fetch_one(pool)
        .await
}

pub async fn get_share_file_from_db(
    external_id: &str,
    pool: &DbPool,
) -> Result<ShareFile, sqlx::Error> {
    sqlx::query_as::<_, ShareFile>(
        "SELECT file_name, mime_type, file_data FROM share_files WHERE external_id=$1",
    )
    .bind(external_id)
    .fetch_one(pool)
    .await
}

pub async fn get_share_file_info_from_db(
    external_id: &str,
    pool: &DbPool,
) -> Result<ShareFile, sqlx::Error> {
    sqlx::query_as::<_, ShareFile>(
        "SELECT file_name, mime_type FROM share_files WHERE external_id=$1",
    )
    .bind(external_id)
    .fetch_one(pool)
    .await
}
