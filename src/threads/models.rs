use crate::utils::tlv::Tlv;
use anyhow::Result;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

const REPLY_TYPE: u8 = 1;

#[derive(Queryable, Insertable, Serialize, Selectable)]
#[diesel(table_name = crate::schema::threads)]
pub struct Thread {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub time: chrono::NaiveDateTime,
    pub reply_data: Vec<u8>, // Bytea maps to Vec<u8>
    pub room_id: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Reply {
    pub id: String,
    pub parent_id: Option<String>,
    pub content: String,
    pub time: chrono::NaiveDateTime,
}

#[derive(Deserialize)]
pub struct NewThread {
    pub title: String,
    pub content: String,
    pub room_id: i32,
}

#[derive(Serialize)]
pub struct ThreadWithReplies {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub time: chrono::NaiveDateTime,
    pub replies: Vec<Reply>,
}

#[derive(Deserialize, Debug)]
pub struct NewReply {
    pub thread_id: i32,
    pub parent_id: Option<String>,
    pub content: String,
}

impl Tlv for Reply {
    fn get_type() -> u8 {
        REPLY_TYPE
    }
    fn encode(&self) -> Result<Vec<u8>> {
        crate::utils::tlv::encode(REPLY_TYPE, self)
    }
    fn decode(data: &[u8]) -> Result<Vec<Self>> {
        if data.is_empty() {
            return Ok(vec![]);
        }
        crate::utils::tlv::decode(data, REPLY_TYPE)
    }
}
