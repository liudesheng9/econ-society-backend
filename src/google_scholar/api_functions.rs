use crate::google_scholar::models::{GoogleScholar, GoogleScholarPubListed, PublicationFilled};
use crate::researcher_card;
use crate::researcher_card::models::{NewResearcherCard, ResearcherCard};
use crate::utils::db::Connection;
use diesel::prelude::*;
use either::Either;
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::serde::json::Json;
use std::env;

pub async fn get_google_scholar(
    scholar_id: &str,
) -> Result<Either<Json<GoogleScholar>, Custom<String>>, Status> {
    let api_url = env::var("GOOGLE_SCHOLAR_API").expect("GOOGLE_SCHOLAR_API must be set");
    let url = format!("{}/author/{}", api_url, scholar_id);

    let response = match reqwest::get(&url).await {
        Ok(resp) => resp,
        Err(_) => return Err(Status::InternalServerError),
    };

    // Check status code
    if response.status().is_success() {
        // Process 200 response
        match response.json::<GoogleScholar>().await {
            Ok(scholar) => Ok(Either::Left(Json(scholar))),
            Err(e) => {
                println!("Error: {:?}", e);
                Err(Status::InternalServerError)
            }
        }
    } else {
        // Process error response (500)
        match response.text().await {
            Ok(error_text) => Ok(Either::Right(Custom(Status::NotFound, error_text))),
            Err(e) => {
                println!("Error: {:?}", e);
                Err(Status::InternalServerError)
            }
        }
    }
}

pub async fn get_google_scolar_publication(
    scholar_id: &str,
    publication_id: &str,
) -> Result<Either<Json<PublicationFilled>, Custom<String>>, Status> {
    let api_url = env::var("GOOGLE_SCHOLAR_API").expect("GOOGLE_SCHOLAR_API must be set");
    let url = format!(
        "{}/author/{}/publication/{}",
        api_url, scholar_id, publication_id
    );

    let response = match reqwest::get(&url).await {
        Ok(resp) => resp,
        Err(_) => return Err(Status::InternalServerError),
    };

    // Check status code
    if response.status().is_success() {
        // Process 200 response
        match response.json::<PublicationFilled>().await {
            Ok(publication) => Ok(Either::Left(Json(publication))),
            Err(e) => {
                println!("Error: {:?}", e);
                Err(Status::InternalServerError)
            }
        }
    } else {
        // Process error response (500)
        match response.text().await {
            Ok(error_text) => Ok(Either::Right(Custom(Status::NotFound, error_text))),
            Err(e) => {
                println!("Error: {:?}", e);
                Err(Status::InternalServerError)
            }
        }
    }
}

#[get("/google_scholar/<scholar_id>")]
pub async fn google_scholar_endpoint(
    scholar_id: &str,
) -> Result<Either<Json<GoogleScholar>, Custom<String>>, Status> {
    get_google_scholar(scholar_id).await
}

#[get("/google_scholar/<scholar_id>/publication/<publication_id>")]
pub async fn google_scholar_publication_endpoint(
    scholar_id: &str,
    publication_id: &str,
) -> Result<Either<Json<PublicationFilled>, Custom<String>>, Status> {
    get_google_scolar_publication(scholar_id, publication_id).await
}

#[post("/google_scholar/update/<scholar_id>")]
pub async fn google_scholar_update_endpoint(
    mut conn: Connection,
    scholar_id: &str,
) -> Result<Json<ResearcherCard>, Status> {
    let scholar = get_google_scholar(scholar_id).await?;

    let scholar = match scholar {
        Either::Left(scholar) => scholar.0,
        Either::Right(_) => return Err(Status::NotFound),
    };

    let scholar_publisted = GoogleScholarPubListed::from_google_scholar(scholar);
    let newresearcher_card = NewResearcherCard::from_google_scholar_pub_listed(scholar_publisted);

    // Check if researcher with this Google Scholar ID already exists
    use crate::schema::researcher_cards::dsl::*;

    match researcher_cards
        .filter(google_scholar_id.eq(&newresearcher_card.google_scholar_id))
        .select(ResearcherCard::as_select())
        .first(&mut conn.0)
    {
        Ok(existing_researcher) => {
            // Found existing researcher, update it
            let updated_researcher = diesel::update(researcher_cards.find(existing_researcher.id))
                .set(&newresearcher_card)
                .returning(ResearcherCard::as_select())
                .get_result(&mut conn.0)
                .map_err(|e| {
                    eprintln!("Error updating researcher card: {}", e);
                    Status::InternalServerError
                })?;

            Ok(Json(updated_researcher))
        }
        Err(diesel::result::Error::NotFound) => {
            // Researcher not found, create new one
            let result = researcher_card::api_functions::create_researcher_card(
                conn,
                Json(newresearcher_card),
            )?;
            Ok(result)
        }
        Err(_) => Err(Status::InternalServerError),
    }
}
