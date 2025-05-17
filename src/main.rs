#[macro_use]
extern crate rocket;
use dotenvy::dotenv;
use rocket::http::Method;
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};

mod google_scholar;
mod researcher_card;
mod researcher_card_threads;
mod schema;
mod threads;
mod utils;

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
        .manage(utils::db::establish_connection())
        .mount("/", routes![index])
        .mount(
            "/api",
            routes![
                threads::api_functions::create_thread,
                threads::api_functions::list_threads_ids,
                threads::api_functions::append_comment,
                threads::api_functions::get_thread,
                researcher_card::api_functions::create_researcher_card,
                researcher_card::api_functions::get_researcher_card,
                researcher_card::api_functions::update_researcher_card,
                researcher_card::api_functions::get_all_researcher_cards,
                researcher_card_threads::api_functions::get_researcher_card_thread,
                researcher_card_threads::api_functions::get_researcher_card_threads,
                researcher_card_threads::api_functions::append_researcher_card_comment,
                researcher_card_threads::api_functions::get_researcher_card_thread_ids,
                researcher_card::api_functions::get_researcher_card_ids,
                google_scholar::api_functions::google_scholar_endpoint,
                google_scholar::api_functions::google_scholar_publication_endpoint,
                google_scholar::api_functions::google_scholar_update_endpoint,
            ],
        )
        .attach(cors)
}
