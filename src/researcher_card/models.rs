use crate::google_scholar::models::GoogleScholarPubListed;
use crate::researcher_card_threads::models::ResearcherCardThread;
use crate::schema::researcher_card_threads::dsl::*;
use crate::utils::db::Connection;
use diesel::prelude::*;
use rocket::http::Status;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Insertable, Serialize, Selectable, Deserialize)]
#[diesel(table_name = crate::schema::researcher_cards)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ResearcherCard {
    pub id: i32,
    pub affiliation: String,
    pub name: String,
    #[serde(default)]
    pub citedby: i32,
    #[serde(default)]
    pub co_authors: Vec<String>,
    #[serde(default)]
    pub email_domain: String,
    #[serde(default)]
    pub interests: Vec<String>,
    #[serde(default)]
    pub google_scholar_publication_ids: Vec<String>,
    #[serde(default)]
    pub google_scholar_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct ResearcherCardComm {
    pub id: i32,
    pub affiliation: String,
    pub name: String,
    #[serde(default)]
    pub citedby: i32,
    #[serde(default)]
    pub email_domain: String,
    #[serde(default)]
    pub interests: Vec<String>,
    #[serde(default)]
    pub google_scholar_publication_ids: Vec<String>,
    #[serde(default)]
    pub google_scholar_id: String,
    pub researcher_card_thread_id: i32,
}

impl ResearcherCardComm {
    pub fn from_researcher_card(
        conn: &mut Connection,
        researcher_card: ResearcherCard,
    ) -> Result<Self, Status> {
        let researcher_card_thread_tar = researcher_card_threads
            .filter(researcher_id.eq(researcher_card.id))
            .first::<ResearcherCardThread>(&mut conn.0)
            .map_err(|_| Status::InternalServerError)?;
        Ok(Self {
            id: researcher_card.id,
            affiliation: researcher_card.affiliation,
            name: researcher_card.name,
            citedby: researcher_card.citedby,
            email_domain: researcher_card.email_domain,
            interests: researcher_card.interests,
            google_scholar_publication_ids: researcher_card.google_scholar_publication_ids,
            google_scholar_id: researcher_card.google_scholar_id,
            researcher_card_thread_id: researcher_card_thread_tar.id,
        })
    }
}

#[derive(Queryable, Insertable, Serialize, Selectable, Deserialize, AsChangeset)]
#[diesel(table_name = crate::schema::researcher_cards)]
pub struct NewResearcherCard {
    pub affiliation: String,
    pub name: String,
    #[serde(default)]
    pub co_authors: Vec<String>,
    #[serde(default)]
    pub citedby: i32,
    #[serde(default)]
    pub email_domain: String,
    #[serde(default)]
    pub interests: Vec<String>,
    #[serde(default)]
    pub google_scholar_publication_ids: Vec<String>,
    #[serde(default)]
    pub google_scholar_id: String,
}

impl NewResearcherCard {
    pub fn from_google_scholar_pub_listed(
        google_scholar_pub_listed: GoogleScholarPubListed,
    ) -> Self {
        Self {
            google_scholar_id: google_scholar_pub_listed.google_scholar_id,
            affiliation: google_scholar_pub_listed.affiliation,
            name: google_scholar_pub_listed.name,
            co_authors: google_scholar_pub_listed.co_authors,
            citedby: google_scholar_pub_listed.citedby,
            email_domain: google_scholar_pub_listed.email_domain,
            interests: google_scholar_pub_listed.interests,
            google_scholar_publication_ids: google_scholar_pub_listed.publications,
        }
    }
}
