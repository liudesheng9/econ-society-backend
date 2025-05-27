use crate::user_mutate::models::UserReduced;
use crate::utils::rds_conn::RdsConn;
use redis::AsyncCommands;
use rocket::http::Status;

/// Save a UserReduced object in Redis with an optional expiration time
pub async fn save_user_reduced_rds(
    mut conn: RdsConn,
    user_reduced: &UserReduced,
    expire_seconds: Option<u64>,
) -> Result<(), Status> {
    // You can adjust this based on your needs
    let expire = expire_seconds.unwrap_or(604800);

    let user_reduced_key = format!("user_reduced:{}", user_reduced.id);

    // Serialize the entire UserReduced struct as JSON
    let serialized =
        serde_json::to_string(&user_reduced).map_err(|_| Status::InternalServerError)?;

    let _: () = conn
        .set_ex(&user_reduced_key, &serialized, expire)
        .await
        .map_err(|_| Status::InternalServerError)?;
    Ok(())
}

/// Get a UserReduced object from Redis
pub async fn get_user_reduced_from_rds(
    mut conn: RdsConn,
    id: i32,
) -> Result<Option<UserReduced>, Status> {
    let user_reduced_key = format!("user_reduced:{}", id);
    let serialized: Option<String> = conn
        .get(&user_reduced_key)
        .await
        .map_err(|_| Status::InternalServerError)?;

    match serialized {
        Some(data) => {
            let user_reduced: UserReduced =
                serde_json::from_str(&data).map_err(|_| Status::InternalServerError)?;
            Ok(Some(user_reduced))
        }
        None => Ok(None),
    }
}

/// Save a UserReduced object with default TTL (convenience function)
pub async fn save_user_reduced_with_default_ttl(
    conn: RdsConn,
    user_reduced: &UserReduced,
) -> Result<(), Status> {
    save_user_reduced_rds(conn, user_reduced, None).await
}

/// Update the TTL of an existing user_reduced key without changing the data
pub async fn refresh_user_reduced_ttl(
    mut conn: RdsConn,
    id: i32,
    expire_seconds: Option<i64>,
) -> Result<(), Status> {
    let expire = expire_seconds.unwrap_or(604800);
    let user_reduced_key = format!("user_reduced:{}", id);

    //check if the key exists
    let key_exists: bool = conn
        .exists(&user_reduced_key)
        .await
        .map_err(|_| Status::InternalServerError)?;

    if !key_exists {
        return Err(Status::NotFound);
    }

    let _: () = conn
        .expire(&user_reduced_key, expire)
        .await
        .map_err(|_| Status::InternalServerError)?;
    Ok(())
}
