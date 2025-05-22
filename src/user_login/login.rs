use crate::schema::current_users::dsl::*;
use crate::user_login::models::{CurrentUser, LoginData, LoginResponse};
use crate::utils::db::Connection;
use crate::utils::token::generate_token;
use diesel::prelude::*;
use rocket::http::Status;
use rocket::serde::json::Json;

#[post("/login", data = "<login_data>")]
pub async fn user_login(
    login_data: Json<LoginData>,
    mut conn: Connection,
) -> Result<Json<LoginResponse>, Status> {
    let login_data = login_data.into_inner();
    // get email and password from login_data
    let user_email = login_data.email;
    let user_password = login_data.password;

    // check if email and password are correct
    let user = current_users
        .filter(email.eq(&user_email))
        .first::<CurrentUser>(&mut conn.0)
        .optional()
        .map_err(|_| Status::InternalServerError)?;
    if user.is_none() {
        return Err(Status::Unauthorized);
    }
    let user = user.unwrap();
    if user.password != user_password {
        return Err(Status::Unauthorized);
    }
    // generate token
    let token = generate_token();
    Ok(Json(LoginResponse { token }))
}
