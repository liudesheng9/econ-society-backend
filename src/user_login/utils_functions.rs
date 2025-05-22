use crate::schema::current_users::dsl::*;
use crate::user_login::models::{CurrentUser, NewCurrentUser};
use crate::utils::db::Connection;
use crate::utils::time::get_current_time;
use diesel::prelude::*;
use rocket::http::Status;

pub async fn create_new_user(
    mut conn: Connection,
    new_user: NewCurrentUser,
) -> Result<CurrentUser, Status> {
    // check if user already exists
    let user = current_users
        .filter(email.eq(&new_user.email))
        .first::<CurrentUser>(&mut conn.0)
        .optional()
        .map_err(|_| Status::InternalServerError)?;

    if user.is_some() {
        return Err(Status::BadRequest);
    }

    let new_user = diesel::insert_into(crate::schema::current_users::table)
        .values((
            name.eq(&new_user.name),
            email.eq(&new_user.email),
            password.eq(&new_user.password),
            created_at.eq(&get_current_time()),
        ))
        .returning(CurrentUser::as_select())
        .get_result(&mut conn.0)
        .map_err(|_| Status::InternalServerError)?;

    Ok(new_user)
}
