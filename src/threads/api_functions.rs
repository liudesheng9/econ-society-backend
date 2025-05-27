use crate::comm_type::models::CommInWrapper;
use crate::rds_mutate::user_token;
use crate::schema::{current_users, threads};
use crate::threads::models::{NewReply, NewThread, Reply, Thread, ThreadWithReplies};
use crate::utils;
use crate::utils::db::Connection;
use crate::utils::rds_conn::RdsConn;
use crate::utils::tlv::Tlv;
use anyhow::Result;
use diesel::prelude::*;
use rocket::http::Status;
use rocket::serde::json::Json;
use std::sync::atomic::{AtomicU16, Ordering};

// Node ID for Snowflake generation
static NODE_ID: AtomicU16 = AtomicU16::new(1);

#[post("/threads", format = "json", data = "<new_thread>")]
pub async fn create_thread(
    rds_conn: RdsConn,
    mut conn: Connection,
    new_thread: Json<CommInWrapper<NewThread>>,
) -> Result<Status, Status> {
    // Create a new thread with an empty data blob
    use crate::schema::threads::dsl::*;

    // Get the user hash from the new_thread
    let user_hash = new_thread.get_user_hash();

    let user_cache_id = user_token::get_user_by_token(rds_conn, user_hash).await?;
    let user_cache_id = user_cache_id.ok_or(Status::Unauthorized)?;

    let user_id_int = user_cache_id.parse::<i32>().map_err(|_| {
        eprintln!("Failed to parse user_id to i32");
        Status::InternalServerError
    })?;

    //check if user id exists
    if !diesel::dsl::select(diesel::dsl::exists(
        current_users::table.filter(current_users::id.eq(user_id_int)),
    ))
    .get_result::<bool>(&mut conn.0)
    .unwrap_or(false)
    {
        return Err(Status::NotFound);
    }

    let new_thread = new_thread.get_data();

    // Insert the thread into the database using a query that omits the id field
    let _ = diesel::insert_into(threads)
        .values((
            title.eq(&new_thread.title),
            content.eq(&new_thread.content),
            reply_data.eq(Vec::<u8>::new()),
            room_id.eq(&new_thread.room_id),
            time.eq(utils::time::get_current_time()),
            user_id.eq(&user_id_int),
        ))
        .returning(Thread::as_select())
        .get_result(&mut conn.0)
        .map_err(|e| {
            eprintln!("Error creating thread: {}", e);
            Status::InternalServerError
        })?;

    Ok(Status::Created)
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

#[post("/comments", format = "json", data = "<new_reply>")]
pub async fn append_comment(
    rds_conn: RdsConn,
    mut conn: Connection,
    new_reply: Json<CommInWrapper<NewReply>>,
) -> Result<Status, Status> {
    // Get the user hash from the new_reply
    let user_hash = new_reply.get_user_hash();

    let user_cache_id = user_token::get_user_by_token(rds_conn, user_hash).await?;
    let user_cache_id = user_cache_id.ok_or(Status::Unauthorized)?;

    let user_id_int = user_cache_id.parse::<i32>().map_err(|_| {
        eprintln!("Failed to parse user_id to i32");
        Status::InternalServerError
    })?;

    //check if user id exists
    if !diesel::dsl::select(diesel::dsl::exists(
        current_users::table.filter(current_users::id.eq(user_id_int)),
    ))
    .get_result::<bool>(&mut conn.0)
    .unwrap_or(false)
    {
        return Err(Status::NotFound);
    }

    // Get the thread from the new_reply
    let new_reply = new_reply.get_data();
    let thread_id = new_reply.thread_id;

    // Check if the thread exists
    if diesel::dsl::select(diesel::dsl::exists(
        threads::table.filter(threads::id.eq(thread_id)),
    ))
    .get_result::<bool>(&mut conn.0)
    .unwrap_or(false)
        == false
    {
        eprintln!("Thread with ID {} not found", thread_id);
        return Err(Status::NotFound);
    }

    let parent_id = new_reply.parent_id.clone();

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
            let replies = if thread.reply_data.is_empty() {
                vec![]
            } else {
                match Reply::decode(&thread.reply_data) {
                    Ok(reply) => reply,
                    Err(_) => vec![],
                }
            };

            // Check if parent_id exists in the replies
            if !replies.iter().any(|r| r.id == parent_id) {
                eprintln!(
                    "Parent reply with ID {} not found in thread {}",
                    parent_id, thread_id
                );
                return Err(Status::NotFound);
            }
        } else {
            return Err(Status::InternalServerError);
        }
    }

    // Generate a unique ID using the Snowflake algorithm
    let node_id = NODE_ID.load(Ordering::SeqCst);
    let reply_id = utils::snowflake::generate_snowflake_id(node_id);

    // Create the reply with the Snowflake ID
    let reply = Reply {
        id: reply_id,
        parent_id: new_reply.parent_id.clone(),
        content: new_reply.content.clone(),
        user_id: user_id_int,
        time: utils::time::get_current_time(),
    };

    // Encode the reply as TLV
    let tlv_data = match reply.encode() {
        Ok(data) => data,
        Err(_) => return Err(Status::InternalServerError),
    };

    // Use a raw SQL query with binary concatenation operator to append data without fetching
    // This uses the PostgreSQL-specific concatenation operator for binary data
    let query = diesel::sql_query("UPDATE threads SET reply_data = reply_data || $1 WHERE id = $2")
        .bind::<diesel::sql_types::Binary, _>(&tlv_data)
        .bind::<diesel::sql_types::Integer, _>(thread_id);

    // Execute the query
    match query.execute(&mut conn.0) {
        Ok(_) => Ok(Status::Created),
        Err(e) => {
            eprintln!("Error appending comment: {}", e);
            Err(Status::InternalServerError)
        }
    }
}

#[get("/thread/<id>")]
pub fn get_thread(mut conn: Connection, id: i32) -> Result<Json<ThreadWithReplies>, Status> {
    // Get the thread from the database
    let thread = threads::table
        .find(id)
        .select(Thread::as_select())
        .first(&mut conn.0)
        .map_err(|_| Status::NotFound)?;

    // Decode the TLV data to get the replies
    let replies = if thread.reply_data.is_empty() {
        vec![]
    } else {
        match Reply::decode(&thread.reply_data) {
            Ok(reply) => reply,
            Err(_) => vec![],
        }
    };

    let thread_with_replies = ThreadWithReplies {
        id: thread.id,
        title: thread.title,
        content: thread.content,
        replies: replies,
        user_id: thread.user_id,
        time: thread.time,
    };

    Ok(Json(thread_with_replies))
}
