use crate::schema::threads;
use crate::threads::models::{
    NewReplyComm, NewThread, Reply, ReplyComm, Thread, ThreadWithRepliesComm,
};
use crate::utils;
use crate::utils::db::Connection;
use crate::utils::tlv::Tlv;
use anyhow::Result;
use diesel::prelude::*;
use rocket::http::Status;
use rocket::serde::json::Json;
use std::sync::atomic::{AtomicU16, Ordering};

// Node ID for Snowflake generation
static NODE_ID: AtomicU16 = AtomicU16::new(1);

#[post("/threads", format = "json", data = "<new_thread>")]
pub fn create_thread(
    mut conn: Connection,
    new_thread: Json<NewThread>,
) -> Result<Json<Thread>, Status> {
    // Create a new thread with an empty data blob
    use crate::schema::threads::dsl::*;

    // Insert the thread into the database using a query that omits the id field
    let thread = diesel::insert_into(threads)
        .values((
            title.eq(&new_thread.title),
            content.eq(&new_thread.content),
            reply_data.eq(Vec::<u8>::new()),
            room_id.eq(&new_thread.room_id),
            time.eq(utils::time::get_current_time()),
        ))
        .returning(Thread::as_select())
        .get_result(&mut conn.0)
        .map_err(|e| {
            eprintln!("Error creating thread: {}", e);
            Status::InternalServerError
        })?;

    Ok(Json(thread))
}

#[get("/threads")]
pub fn list_threads_ids(mut conn: Connection) -> Result<Json<Vec<i32>>, Status> {
    // Get all threads from the database
    let threads = threads::table
        .select(Thread::as_select())
        .load(&mut conn.0)
        .map_err(|_| Status::InternalServerError)?;

    // Print all thread IDs
    println!("Listing all threads:");
    for thread in &threads {
        println!("  Thread ID: {}, Title: {}", thread.id, thread.title);
    }

    // get thread id list
    let thread_ids = threads.iter().map(|t| t.id).collect::<Vec<i32>>();
    Ok(Json(thread_ids))
}

#[post("/comments", format = "json", data = "<new_reply_comm>")]
pub fn append_comment(mut conn: Connection, new_reply_comm: Json<NewReplyComm>) -> Status {
    // Get the thread from the new_reply
    let thread_id = new_reply_comm.thread_id;

    // Convert parent_id from Option<String> to Option<i64>
    let parent_id = match &new_reply_comm.parent_id {
        Some(id_str) => match id_str.parse::<i64>() {
            Ok(id) => Some(id),
            Err(_) => {
                eprintln!("Invalid parent_id format: {}", id_str);
                return Status::BadRequest;
            }
        },
        None => None,
    };

    // Check if the thread exists
    if diesel::dsl::select(diesel::dsl::exists(
        threads::table.filter(threads::id.eq(thread_id)),
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
        let thread = threads::table
            .find(thread_id)
            .select(Thread::as_select())
            .first(&mut conn.0)
            .map_err(|_| {
                eprintln!("Failed to fetch thread {}", thread_id);
                Status::InternalServerError
            });

        if let Ok(thread) = thread {
            let replies = match Reply::decode(&thread.reply_data) {
                Ok(reply) => vec![reply],
                Err(_) => vec![],
            };

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
    let tlv_data = match reply.encode() {
        Ok(data) => data,
        Err(_) => return Status::InternalServerError,
    };

    // Use a raw SQL query with binary concatenation operator to append data without fetching
    // This uses the PostgreSQL-specific concatenation operator for binary data
    let query = diesel::sql_query("UPDATE threads SET reply_data = reply_data || $1 WHERE id = $2")
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

#[get("/thread/<id>")]
pub fn get_thread(mut conn: Connection, id: i32) -> Result<Json<ThreadWithRepliesComm>, Status> {
    // Get the thread from the database
    let thread = threads::table
        .find(id)
        .select(Thread::as_select())
        .first(&mut conn.0)
        .map_err(|_| Status::NotFound)?;

    // Decode the TLV data to get the replies
    let replies = match Reply::decode(&thread.reply_data) {
        Ok(reply) => vec![reply],
        Err(_) => vec![],
    };

    // Convert internal Reply objects to ReplyComm objects for frontend communication
    let comm_replies = ReplyComm::from_replies(replies);

    let thread_with_replies = ThreadWithRepliesComm {
        id: thread.id,
        title: thread.title,
        content: thread.content,
        replies: comm_replies,
        time: thread.time,
    };

    Ok(Json(thread_with_replies))
}
