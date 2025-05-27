use crate::rds_mutate::user_token::{get_token_by_user, store_user_token};
use crate::schema::current_users::dsl::*;
use crate::user_mutate::models::{CurrentUser, LoginData, LoginResponse};
use crate::utils::db::Connection;
use crate::utils::random_hashers::RandomHasher;
use crate::utils::rds_conn::RdsConn;
use diesel::prelude::*;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;

pub async fn find_or_add_user_hash(
    rds_conn: RdsConn,
    user_id: i32,
    random_hasher: &State<RandomHasher>,
) -> Result<String, Status> {
    let token = get_token_by_user(rds_conn.clone(), &user_id.to_string()).await?;
    if token.is_some() {
        return Ok(token.unwrap());
    }
    let hash = random_hasher.hash_with_salt(&format!("econ_{}", &user_id.to_string()));
    // store the token in redis with an expiration time of 24 hours
    store_user_token(rds_conn, &user_id.to_string(), &hash, Some(86400)).await?;
    Ok(hash)
}

#[post("/login", data = "<login_data>")]
pub async fn user_login(
    login_data: Json<LoginData>,
    mut conn: Connection,
    rds_conn: RdsConn,
    random_hasher: &State<RandomHasher>,
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

    let user_id = user.id;
    let hash = find_or_add_user_hash(rds_conn, user_id, random_hasher).await?;
    Ok(Json(LoginResponse { hash }))
}
