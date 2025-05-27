use crate::comm_type::models::CommInWrapper;
use crate::rds_mutate::user_token::get_user_by_token;
use crate::researcher_card::api_functions::whether_card_is_exist;
use crate::researcher_card_threads::models::{
    NewResearcherCardThread, ResearcherCardThread, ResearcherCardThreadWithReplies,
};
use crate::schema;
use crate::threads::models::{NewReply, Reply};
use crate::utils;
use crate::utils::db::Connection;
use crate::utils::rds_conn::RdsConn;
use crate::utils::tlv::Tlv;
use diesel::prelude::*;
use rocket::http::Status;
use rocket::serde::json::Json;
use std::sync::atomic::{AtomicU16, Ordering};

static NODE_ID: AtomicU16 = AtomicU16::new(1);

pub fn whether_researcher_card_thread_is_exist(
    conn: &mut Connection,
    thread_id: i32,
) -> Result<bool, Status> {
    let _ = schema::researcher_card_threads::table
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
    if schema::researcher_card_threads::table
        .filter(schema::researcher_card_threads::researcher_id.eq(new_thread.researcher_id))
        .first::<ResearcherCardThread>(&mut conn.0)
        .is_ok()
    {
        return Err(Status::BadRequest);
    }

    // generate create time for the thread
    new_thread.time = utils::time::get_current_time();

    // Create a new thread with an empty data blob
    let thread = diesel::insert_into(schema::researcher_card_threads::table)
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
) -> Result<Json<Vec<ResearcherCardThreadWithReplies>>, Status> {
    let threads = schema::researcher_card_threads::table
        .load::<ResearcherCardThread>(&mut conn.0)
        .map_err(|_| Status::InternalServerError)?;
    let threads_with_replies = threads
        .iter()
        .map(|thread| ResearcherCardThreadWithReplies::from_database_thread(thread))
        .collect::<Vec<ResearcherCardThreadWithReplies>>();
    Ok(Json(threads_with_replies))
}

#[get("/threads/researcher_card/ids")]
pub fn get_researcher_card_thread_ids(mut conn: Connection) -> Result<Json<Vec<i32>>, Status> {
    let threads = schema::researcher_card_threads::table
        .load::<ResearcherCardThread>(&mut conn.0)
        .map_err(|_| Status::InternalServerError)?;
    let thread_ids = threads.iter().map(|t| t.id).collect::<Vec<i32>>();
    Ok(Json(thread_ids))
}

#[get("/threads/researcher_card/<thread_id>")]
pub fn get_researcher_card_thread(
    mut conn: Connection,
    thread_id: i32,
) -> Result<Json<ResearcherCardThreadWithReplies>, Status> {
    // check whether the thread is exist
    if !whether_researcher_card_thread_is_exist(&mut conn, thread_id)? {
        return Err(Status::NotFound);
    }

    let thread = schema::researcher_card_threads::table
        .find(thread_id)
        .first::<ResearcherCardThread>(&mut conn.0)
        .map_err(|_| Status::InternalServerError)?;
    let thread_with_replies = ResearcherCardThreadWithReplies::from_database_thread(&thread);

    Ok(Json(thread_with_replies))
}
#[post("/comments/researcher_card", format = "json", data = "<new_reply>")]
pub async fn append_researcher_card_comment(
    rds_conn: RdsConn,
    mut conn: Connection,
    new_reply: Json<CommInWrapper<NewReply>>,
) -> Result<Status, Status> {
    // Get the user hash from the new_reply
    let user_hash = new_reply.get_user_hash();

    let user_cache_id = get_user_by_token(rds_conn, user_hash).await?;
    let user_cache_id = user_cache_id.ok_or(Status::Unauthorized)?;

    let user_id_int = user_cache_id.parse::<i32>().map_err(|_| {
        eprintln!("Failed to parse user_id to i32");
        Status::InternalServerError
    })?;

    //check if user id exists
    if !diesel::dsl::select(diesel::dsl::exists(
        schema::current_users::table.filter(schema::current_users::id.eq(user_id_int)),
    ))
    .get_result::<bool>(&mut conn.0)
    .unwrap_or(false)
    {
        return Err(Status::NotFound);
    }

    // Get the thread from the new_reply
    let new_reply = new_reply.get_data();
    let thread_id = new_reply.thread_id;

    let parent_id = new_reply.parent_id.clone();

    // Check if the thread exists
    if diesel::dsl::select(diesel::dsl::exists(
        schema::researcher_card_threads::table
            .filter(schema::researcher_card_threads::id.eq(thread_id)),
    ))
    .get_result::<bool>(&mut conn.0)
    .unwrap_or(false)
        == false
    {
        eprintln!("Thread with ID {} not found", thread_id);
        return Err(Status::NotFound);
    }

    // Validate parent_id
    // For thread-level replies, parent_id should be None
    // If parent_id is specified, check that it exists in the thread
    if let Some(parent_id) = parent_id {
        let thread = schema::researcher_card_threads::table
            .find(thread_id)
            .first::<ResearcherCardThread>(&mut conn.0)
            .map_err(|_| {
                eprintln!("Failed to fetch thread {}", thread_id);
                Status::InternalServerError
            });

        if let Ok(thread) = thread {
            let replies = match Reply::decode(&thread.reply_data) {
                Ok(replies) => replies,
                Err(_) => vec![],
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
        time: utils::time::get_current_time(),
        user_id: user_id_int,
    };

    println!("reply: {:?}", reply);

    // Encode the reply as TLV
    let tlv_data = match reply.encode() {
        Ok(tlv_data) => tlv_data,
        Err(_) => return Err(Status::InternalServerError),
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
        Ok(_) => Ok(Status::Created),
        Err(e) => {
            eprintln!("Error appending comment: {}", e);
            Err(Status::InternalServerError)
        }
    }
}
