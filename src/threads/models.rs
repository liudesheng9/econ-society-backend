use crate::utils::tlv;
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
    pub id: i64,
    pub parent_id: Option<i64>,
    pub content: String,
    pub time: chrono::NaiveDateTime,
}

#[derive(Deserialize)]
pub struct NewThread {
    pub title: String,
    pub content: String,
    pub room_id: i32,
}

#[derive(Deserialize)]
pub struct NewReplyComm {
    pub thread_id: i32,
    pub parent_id: Option<String>,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReplyComm {
    pub id: String,
    pub parent_id: Option<String>,
    pub content: String,
    pub time: chrono::NaiveDateTime,
}

#[derive(Serialize)]
pub struct ThreadWithRepliesComm {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub time: chrono::NaiveDateTime,
    pub replies: Vec<ReplyComm>,
}

#[derive(Deserialize, Debug)]
pub struct NewReply {
    pub thread_id: i32,
    pub parent_id: Option<i64>,
    pub content: String,
}

// Conversion methods
impl ReplyComm {
    // Convert internal Reply to communication ReplyComm
    pub fn from_reply(reply: Reply) -> Self {
        ReplyComm {
            id: reply.id.to_string(),
            parent_id: reply.parent_id.map(|pid| pid.to_string()),
            content: reply.content,
            time: reply.time,
        }
    }

    // Convert a vector of Reply objects to a vector of ReplyComm objects
    pub fn from_replies(replies: Vec<Reply>) -> Vec<Self> {
        replies.into_iter().map(Self::from_reply).collect()
    }
}

impl Tlv for Reply {
    fn get_type() -> u8 {
        REPLY_TYPE
    }
    fn encode(&self) -> Result<Vec<u8>> {
        crate::utils::tlv::encode(REPLY_TYPE, self)
    }
    fn decode(data: &[u8]) -> Result<Self> {
        let decoded = crate::utils::tlv::decode(data, REPLY_TYPE)?;
        Ok(decoded.into_iter().next().unwrap())
    }
}
