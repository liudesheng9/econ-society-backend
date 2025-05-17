use crate::researcher_card::api_functions::whether_card_is_exist;
use crate::researcher_card_threads::models::{
    NewResearcherCardThread, ResearcherCardThread, ResearcherCardThreadWithRepliesComm,
};
use crate::schema::researcher_card_threads::dsl::*;
use crate::threads::models::{NewReplyComm, Reply};
use crate::utils;
use crate::utils::db::Connection;
use crate::utils::tlv;
use diesel::prelude::*;
use rocket::http::Status;
use rocket::serde::json::Json;
use std::sync::atomic::{AtomicU16, Ordering};

static NODE_ID: AtomicU16 = AtomicU16::new(1);

pub fn whether_researcher_card_thread_is_exist(
    conn: &mut Connection,
    thread_id: i32,
) -> Result<bool, Status> {
    let _ = researcher_card_threads
        .find(thread_id)
        .first::<ResearcherCardThread>(&mut conn.0)
        .map_err(|_| Status::InternalServerError)?;
    Ok(true)
}

pub fn create_researcher_card_thread(
    mut conn: Connection,
    mut new_thread: Json<NewResearcherCardThread>,
) -> Result<Json<ResearcherCardThread>, Status> {
    // check if the researcher card is exist
    if !whether_card_is_exist(&mut conn, new_thread.researcher_id)? {
        return Err(Status::NotFound);
    }

    //check if there is another researcher thread use the same researcher card
    if researcher_card_threads
        .filter(researcher_id.eq(new_thread.researcher_id))
        .first::<ResearcherCardThread>(&mut conn.0)
        .is_ok()
    {
        return Err(Status::BadRequest);
    }

    // generate create time for the thread
    new_thread.time = utils::time::get_current_time();

    // Create a new thread with an empty data blob
    let thread = diesel::insert_into(researcher_card_threads)
        .values(&*new_thread)
        .returning(ResearcherCardThread::as_select())
        .get_result(&mut conn.0)
        .map_err(|e| {
            eprintln!("Error creating thread: {}", e);
            Status::InternalServerError
        })?;

    Ok(Json(thread))
}

#[get("/threads/researcher_card")]
pub fn get_researcher_card_threads(
    mut conn: Connection,
) -> Result<Json<Vec<ResearcherCardThreadWithRepliesComm>>, Status> {
    let threads = researcher_card_threads
        .load::<ResearcherCardThread>(&mut conn.0)
        .map_err(|_| Status::InternalServerError)?;
    let threads_with_replies = threads
        .iter()
        .map(|thread| ResearcherCardThreadWithRepliesComm::from_database_thread(thread))
        .collect::<Vec<ResearcherCardThreadWithRepliesComm>>();
    Ok(Json(threads_with_replies))
}

#[get("/threads/researcher_card/ids")]
pub fn get_researcher_card_thread_ids(mut conn: Connection) -> Result<Json<Vec<i32>>, Status> {
    let threads = researcher_card_threads
        .load::<ResearcherCardThread>(&mut conn.0)
        .map_err(|_| Status::InternalServerError)?;
    let thread_ids = threads.iter().map(|t| t.id).collect::<Vec<i32>>();
    Ok(Json(thread_ids))
}

#[get("/threads/researcher_card/<thread_id>")]
pub fn get_researcher_card_thread(
    mut conn: Connection,
    thread_id: i32,
) -> Result<Json<ResearcherCardThreadWithRepliesComm>, Status> {
    // check whether the thread is exist
    if !whether_researcher_card_thread_is_exist(&mut conn, thread_id)? {
        return Err(Status::NotFound);
    }

    let thread = researcher_card_threads
        .find(thread_id)
        .first::<ResearcherCardThread>(&mut conn.0)
        .map_err(|_| Status::InternalServerError)?;
    let thread_with_replies = ResearcherCardThreadWithRepliesComm::from_database_thread(&thread);

    Ok(Json(thread_with_replies))
}
#[post(
    "/comments/researcher_card",
    format = "json",
    data = "<new_reply_comm>"
)]
pub fn append_researcher_card_comment(
    mut conn: Connection,
    new_reply_comm: Json<NewReplyComm>,
) -> Status {
    // Get the thread from the new_reply
    let thread_id = new_reply_comm.thread_id;

    // Convert parent_id from Option<String> to Option<i64>
    let parent_id = match &new_reply_comm.parent_id {
        Some(id_str) => match id_str.parse::<i64>() {
            Ok(parsed_id) => Some(parsed_id),
            Err(_) => {
                eprintln!("Invalid parent_id format: {}", id_str);
                return Status::BadRequest;
            }
        },
        None => None,
    };

    // Check if the thread exists
    if diesel::dsl::select(diesel::dsl::exists(
        researcher_card_threads.filter(id.eq(thread_id)),
    ))
    .get_result::<bool>(&mut conn.0)
    .unwrap_or(false)
        == false
    {
        eprintln!("Thread with ID {} not found", thread_id);
        return Status::NotFound;
    }

    // Validate parent_id
    // For thread-level replies, parent_id should be None
    // If parent_id is specified, check that it exists in the thread
    if let Some(parent_id) = parent_id {
        let thread = researcher_card_threads
            .find(thread_id)
            .first::<ResearcherCardThread>(&mut conn.0)
            .map_err(|_| {
                eprintln!("Failed to fetch thread {}", thread_id);
                Status::InternalServerError
            });

        if let Ok(thread) = thread {
            let replies = tlv::decode_replies(&thread.reply_data).unwrap_or_default();

            // Check if parent_id exists in the replies
            if !replies.iter().any(|r| r.id == parent_id) {
                eprintln!(
                    "Parent reply with ID {} not found in thread {}",
                    parent_id, thread_id
                );
                return Status::NotFound;
            }
        } else {
            return Status::InternalServerError;
        }
    }

    // Generate a unique ID using the Snowflake algorithm
    let node_id = NODE_ID.load(Ordering::SeqCst);
    let reply_id = utils::snowflake::generate_snowflake_id(node_id);

    // Create the reply with the Snowflake ID
    let reply = Reply {
        id: reply_id,
        parent_id,
        content: new_reply_comm.content.clone(),
        time: utils::time::get_current_time(),
    };

    // Encode the reply as TLV
    let tlv_data = match tlv::encode_reply(&reply) {
        Ok(tlv_data) => tlv_data,
        Err(_) => return Status::InternalServerError,
    };

    // Use a raw SQL query with binary concatenation operator to append data without fetching
    // This uses the PostgreSQL-specific concatenation operator for binary data
    let query = diesel::sql_query(
        "UPDATE researcher_card_threads SET reply_data = reply_data || $1 WHERE id = $2",
    )
    .bind::<diesel::sql_types::Binary, _>(&tlv_data)
    .bind::<diesel::sql_types::Integer, _>(thread_id);

    // Execute the query
    match query.execute(&mut conn.0) {
        Ok(_) => Status::Created,
        Err(e) => {
            eprintln!("Error appending comment: {}", e);
            Status::InternalServerError
        }
    }
}
