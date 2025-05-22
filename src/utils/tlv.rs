use anyhow::Result;
use bincode;
use serde;
use std::io::{Cursor, Read};

const REPLY_TYPE: u8 = 1;

pub fn encode<T: serde::Serialize>(value_type: u8, data: &T) -> Result<Vec<u8>> {
    // Serialize the data to binary
    let value = bincode::serialize(data)?;
    let length = value.len() as u32;

    let mut tlv = Vec::new();
    tlv.push(value_type); // Type
    tlv.extend_from_slice(&length.to_be_bytes()); // Length (big-endian)
    tlv.extend_from_slice(&value); // Value
    Ok(tlv)
}

pub fn decode<T: serde::de::DeserializeOwned>(blob: &[u8], expected_type: u8) -> Result<Vec<T>> {
    let mut items = Vec::new();
    let mut cursor = Cursor::new(blob);

    while cursor.position() < blob.len() as u64 {
        let t = blob[cursor.position() as usize];
        cursor.set_position(cursor.position() + 1);

        if t != expected_type {
            continue; // Skip unknown types
        }

        let mut len_bytes = [0u8; 4];
        cursor.read_exact(&mut len_bytes)?;
        let len = u32::from_be_bytes(len_bytes) as usize;

        let mut value_bytes = vec![0u8; len];
        cursor.read_exact(&mut value_bytes)?;
        let item: T = bincode::deserialize(&value_bytes)?;
        items.push(item);
    }
    Ok(items)
}

pub trait Tlv {
    fn get_type() -> u8;
    fn encode(&self) -> Result<Vec<u8>>;
    fn decode(data: &[u8]) -> Result<Vec<Self>>
    where
        Self: Sized;
}
