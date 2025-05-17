use crate::researcher_card::models::{NewResearcherCard, ResearcherCard, ResearcherCardComm};
use crate::researcher_card_threads::api_functions::create_researcher_card_thread;
use crate::researcher_card_threads::models::NewResearcherCardThread;
use crate::schema::researcher_cards;
use crate::utils;
use crate::utils::db::Connection;
use diesel::prelude::*;
use rocket::http::Status;
use rocket::serde::json::Json;

pub fn whether_card_is_exist(conn: &mut Connection, id: i32) -> Result<bool, Status> {
    let _ = researcher_cards::table
        .find(id)
        .select(ResearcherCard::as_select())
        .first(&mut conn.0)
        .map_err(|_| Status::InternalServerError)?;
    Ok(true)
}

#[post("/researcher_card", format = "json", data = "<new_researcher_card>")]
pub fn create_researcher_card(
    mut conn: Connection,
    new_researcher_card: Json<NewResearcherCard>,
) -> Result<Json<ResearcherCard>, Status> {
    use crate::schema::researcher_cards::dsl::*;

    let researcher_card = diesel::insert_into(researcher_cards)
        .values(&*new_researcher_card)
        .returning(ResearcherCard::as_select())
        .get_result(&mut conn.0)
        .map_err(|e| {
            eprintln!("Error creating researcher card: {}", e);
            Status::InternalServerError
        })?;

    // create a thread for the researcher card
    create_researcher_card_thread(
        conn,
        Json(NewResearcherCardThread {
            title: "".to_string(),
            content: "".to_string(),
            researcher_id: researcher_card.id,
            time: utils::time::get_current_time(),
        }),
    )
    .map_err(|e| {
        eprintln!("Error creating researcher card thread: {}", e);
        Status::InternalServerError
    })?;
    Ok(Json(researcher_card))
}

#[get("/researcher_card/ids")]
pub fn get_researcher_card_ids(mut conn: Connection) -> Result<Json<Vec<i32>>, Status> {
    let researcher_cards = researcher_cards::table
        .select(researcher_cards::id)
        .load(&mut conn.0)
        .map_err(|_| Status::InternalServerError)?;
    Ok(Json(researcher_cards))
}

#[get("/researcher_card/<id>")]
pub fn get_researcher_card(
    mut conn: Connection,
    id: i32,
) -> Result<Json<ResearcherCardComm>, Status> {
    // check if the card is exist
    if !whether_card_is_exist(&mut conn, id)? {
        return Err(Status::NotFound);
    }
    let researcher_card = researcher_cards::table
        .find(id)
        .select(ResearcherCard::as_select())
        .first(&mut conn.0)
        .map_err(|e| {
            eprintln!("Error getting researcher card: {}", e);
            Status::InternalServerError
        })?;

    let researcher_card_comm =
        ResearcherCardComm::from_researcher_card(&mut conn, researcher_card)?;
    Ok(Json(researcher_card_comm))
}

#[get("/researcher_card/all")]
pub fn get_all_researcher_cards(mut conn: Connection) -> Result<Json<Vec<ResearcherCard>>, Status> {
    let researcher_cards = researcher_cards::table
        .select(ResearcherCard::as_select())
        .load(&mut conn.0)
        .map_err(|_| Status::InternalServerError)?;
    Ok(Json(researcher_cards))
}

#[post(
    "/researcher_card/<id>/update",
    format = "json",
    data = "<new_researcher_card>"
)]
pub fn update_researcher_card(
    mut conn: Connection,
    id: i32,
    new_researcher_card: Json<NewResearcherCard>,
) -> Result<Json<ResearcherCard>, Status> {
    // check if the card is exist
    if !whether_card_is_exist(&mut conn, id)? {
        return Err(Status::NotFound);
    }
    let researcher_card = diesel::update(researcher_cards::table.find(id))
        .set(&*new_researcher_card)
        .returning(ResearcherCard::as_select())
        .get_result(&mut conn.0)
        .map_err(|e| {
            eprintln!("Error updating researcher card: {}", e);
            Status::InternalServerError
        })?;
    Ok(Json(researcher_card))
}
