use crate::domain::error::GatewayError;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::{Read, Write};

pub fn compress_data(data: &[u8]) -> Result<Vec<u8>, GatewayError> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(data)
        .map_err(|e| GatewayError::CompressionError(e.to_string()))?;
    encoder
        .finish()
        .map_err(|e| GatewayError::CompressionError(e.to_string()))
}

pub fn decompress_data(data: &[u8]) -> Result<Vec<u8>, GatewayError> {
    let mut decoder = ZlibDecoder::new(data);
    let mut decompressed_data = Vec::new();
    decoder
        .read_to_end(&mut decompressed_data)
        .map_err(|e| GatewayError::DecompressionError(e.to_string()))?;
    Ok(decompressed_data)
}
