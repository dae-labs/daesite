use crate::domain::error::WebSocketError;
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use std::io::{Read, Write};

pub fn compress_data(data: &[u8]) -> Result<Vec<u8>, WebSocketError> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    Ok(encoder.finish()?)
}

pub fn decompress_data(data: &[u8]) -> Result<Vec<u8>, WebSocketError> {
    let mut decoder = ZlibDecoder::new(data);
    let mut decompressed_data = Vec::new();
    decoder.read_to_end(&mut decompressed_data)?;
    Ok(decompressed_data)
}
