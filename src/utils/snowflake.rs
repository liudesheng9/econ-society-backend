use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use std::sync::atomic::{AtomicU16, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

// Constants for Snowflake ID generation
const EPOCH_OFFSET: u64 = 1609459200000; // Custom epoch (e.g., Jan 1, 2021)
const NODE_ID_BITS: u8 = 10;
const SEQUENCE_BITS: u8 = 12;

// Static counter for the sequence portion of the snowflake
static SEQUENCE: AtomicU16 = AtomicU16::new(0);

// Generate a unique snowflake ID based on:
// - 41 bits: timestamp (milliseconds since custom epoch)
// - 10 bits: node ID (can be configured for distributed systems)
pub fn generate_snowflake_id(node_id: u16) -> String {
    // Make sure node_id is within valid range
    let node_id = node_id & ((1 << NODE_ID_BITS) - 1);

    // Get current timestamp in milliseconds
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as u64;

    // Calculate milliseconds since custom epoch
    let timestamp_since_epoch = timestamp - EPOCH_OFFSET;

    // Get next sequence number
    let sequence = SEQUENCE.fetch_add(1, Ordering::SeqCst) & ((1 << SEQUENCE_BITS) - 1);

    // Combine parts to form the Snowflake ID
    let snowflake_id = ((timestamp_since_epoch << (NODE_ID_BITS + SEQUENCE_BITS))
        | ((node_id as u64) << SEQUENCE_BITS)
        | (sequence as u64)) as i64;

    let bytes = snowflake_id.to_be_bytes();
    let base64_str = URL_SAFE.encode(bytes);
    base64_str
}
