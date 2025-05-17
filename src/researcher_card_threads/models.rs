use crate::threads::models::{Reply, ReplyComm};
use crate::utils::tlv::Tlv;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

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
        let replies = match Reply::decode(&thread.reply_data) {
            Ok(reply) => vec![reply],
            Err(_) => vec![],
        };
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
