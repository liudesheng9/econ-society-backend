use crate::tlv;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
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

#[derive(Queryable, Insertable, Serialize, Selectable)]
#[diesel(table_name = crate::schema::researcher_card_threads)]
pub struct ResearcherCardThread {
    pub id: i32,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub content: String,
    pub time: chrono::NaiveDateTime,
    pub reply_data: Vec<u8>, // Bytea maps to Vec<u8>
    pub researcher_id: i32,
}

#[derive(Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::researcher_card_threads)]
pub struct NewResearcherCardThread {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub content: String,
    pub researcher_id: i32,
    #[serde(default)]
    pub time: chrono::NaiveDateTime,
}

#[derive(Serialize)]
pub struct ResearcherCardThreadWithRepliesComm {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub time: chrono::NaiveDateTime,
    pub replies: Vec<ReplyComm>,
    pub researcher_id: i32,
}

impl ResearcherCardThreadWithRepliesComm {
    pub fn from_database_thread(thread: &ResearcherCardThread) -> Self {
        let replies = tlv::decode_replies(&thread.reply_data).unwrap_or_default();
        let comm_replies = ReplyComm::from_replies(replies);
        Self {
            id: thread.id,
            title: thread.title.clone(),
            content: thread.content.clone(),
            time: thread.time,
            replies: comm_replies,
            researcher_id: thread.researcher_id,
        }
    }
}
