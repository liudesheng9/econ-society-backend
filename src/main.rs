#[macro_use]
extern crate rocket;
use dotenvy::dotenv;
use rocket::http::Method;
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};

mod api;
mod db;
mod google_scholar;
mod models;
mod schema;
mod snowflake;
mod tlv;
mod utils;

use api::{
    append_comment, create_thread, get_thread, list_threads_ids,
    researcher_card::{
        create_researcher_card, get_all_researcher_cards, get_researcher_card,
        get_researcher_card_ids, update_researcher_card,
    },
    researcher_card_threads::{
        append_researcher_card_comment, get_researcher_card_thread, get_researcher_card_thread_ids,
        get_researcher_card_threads,
    },
};
use google_scholar::{
    google_scholar_endpoint, google_scholar_publication_endpoint, google_scholar_update_endpoint,
};

#[get("/")]
fn index() -> &'static str {
    "Community Web API"
}

#[launch]
fn rocket() -> _ {
    dotenv().ok();

    // Configure CORS
    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_methods(
            vec![
                Method::Get,
                Method::Post,
                Method::Put,
                Method::Delete,
                Method::Options,
            ]
            .into_iter()
            .map(From::from)
            .collect(),
        )
        .allowed_headers(AllowedHeaders::all())
        .allow_credentials(true)
        .to_cors()
        .expect("CORS configuration error");

    // Mount the routes
    rocket::build()
        .manage(db::establish_connection())
        .mount("/", routes![index])
        .mount(
            "/api",
            routes![
                create_thread,
                list_threads_ids,
                append_comment,
                get_thread,
                create_researcher_card,
                get_researcher_card,
                update_researcher_card,
                google_scholar_endpoint,
                google_scholar_publication_endpoint,
                get_all_researcher_cards,
                google_scholar_update_endpoint,
                get_researcher_card_thread,
                get_researcher_card_threads,
                append_researcher_card_comment,
                get_researcher_card_thread_ids,
                get_researcher_card_ids,
            ],
        )
        .attach(cors)
}
