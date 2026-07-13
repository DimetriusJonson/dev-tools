use std::io::Write;

use flate2::{
    Compression,
    write::{GzDecoder, GzEncoder},
};

pub fn compress_bytes(data: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    let compressed_bytes = encoder.finish()?;
    Ok(compressed_bytes)
}

pub fn decompress_bytes(data: Vec<u8>) -> std::io::Result<Vec<u8>> {
    let mut writer = Vec::new();
    let mut decoder = GzDecoder::new(writer);
    decoder.write_all(&data)?;
    writer = decoder.finish()?;

    Ok(writer)
}
