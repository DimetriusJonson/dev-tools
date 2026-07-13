use std::io::Cursor;

use image::{ImageFormat, codecs::jpeg::JpegEncoder};

pub fn create_image_thumbnail(
    src: &[u8],
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

pub fn convert_image_data_to_jpg(data: &[u8]) -> Result<Vec<u8>, image::ImageError> {
    let img = image::load_from_memory(data)?;

    let mut dst = Vec::new();
    let mut cursor = Cursor::new(&mut dst);

    let mut encoder = JpegEncoder::new_with_quality(&mut cursor, 80);
    encoder.encode_image(&img)?;

    Ok(dst)
}

