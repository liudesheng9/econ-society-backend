use crate::rds_mutate::user_reduced::{get_user_reduced_from_rds, save_user_reduced_rds};
use crate::schema::current_users;
use crate::schema::current_users::dsl::*;
use crate::user_mutate::models::{CurrentUser, UserReduced};
use crate::utils::db::Connection;
use crate::utils::rds_conn::RdsConn;
use diesel::prelude::*;

use rocket::http::Status;
use rocket::serde::json::Json;

pub async fn get_user_reduced_from_db(
    user_id: i32,
    mut conn: Connection,
) -> Result<UserReduced, Status> {
    let user_data = current_users
        .find(user_id)
        .select(CurrentUser::as_select())
        .first(&mut conn.0)
        .map_err(|_| Status::NotFound)?;
    let user_reduced = UserReduced::from(user_data);
    Ok(user_reduced)
}

#[get("/user/reduced/<user_id>")]
pub async fn get_user_reduced(
    user_id: i32,
    rdsconn: RdsConn,
    pgconn: Connection,
) -> Result<Json<UserReduced>, Status> {
    let rdsconn_cloned = rdsconn.clone();
    let user_reduced = get_user_reduced_from_rds(rdsconn, user_id).await?;
    match user_reduced {
        Some(user_reduced) => Ok(Json(user_reduced)),
        None => {
            // search in the database
            let user_reduced = get_user_reduced_from_db(user_id, pgconn).await?;
            save_user_reduced_rds(rdsconn_cloned, &user_reduced, Some(604800)).await?;
            Ok(Json(user_reduced))
        }
    }
}
