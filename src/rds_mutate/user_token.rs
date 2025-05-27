use redis::AsyncCommands;
use rocket::http::Status;

use crate::utils::rds_conn::RdsConn;

/// Store a user token in Redis with an expiration time
pub async fn store_user_token(
    mut conn: RdsConn,
    user_id: &str,
    token: &str,
    expire_seconds: Option<u64>,
) -> Result<(), Status> {
    // Default expiration of 24 hours if not specified
    let expire = expire_seconds.unwrap_or(86400);

    // Store user -> token mapping
    let user_key = format!("user:{}:token", user_id);
    let _: () = conn
        .set_ex(&user_key, token, expire)
        .await
        .map_err(|_| Status::InternalServerError)?;

    // Store token -> user_id mapping for lookup
    let token_key = format!("token:{}", token);
    let _: () = conn
        .set_ex(&token_key, user_id, expire)
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(())
}

/// Get user ID by token
pub async fn get_user_by_token(mut conn: RdsConn, token: &str) -> Result<Option<String>, Status> {
    let token_key = format!("token:{}", token);
    let user_id: Option<String> = conn
        .get(&token_key)
        .await
        .map_err(|_| Status::InternalServerError)?;
    Ok(user_id)
}

/// Get token by user IDs
pub async fn get_token_by_user(mut conn: RdsConn, user_id: &str) -> Result<Option<String>, Status> {
    let user_key = format!("user:{}:token", user_id);
    let token: Option<String> = conn
        .get(&user_key)
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(token)
}

/// Remove user token from Redis
pub async fn remove_user_token(mut conn: RdsConn, user_id: &str) -> Result<(), Status> {
    // First get the token to remove the reverse mapping
    let user_key = format!("user:{}:token", user_id);
    let token: Option<String> = conn
        .get(&user_key)
        .await
        .map_err(|_| Status::InternalServerError)?;

    // Delete the user -> token mapping
    let _: () = conn
        .del(&user_key)
        .await
        .map_err(|_| Status::InternalServerError)?;

    // If we found a token, delete the token -> user mapping as well
    if let Some(token) = token {
        let token_key = format!("token:{}", token);
        let _: () = conn
            .del(&token_key)
            .await
            .map_err(|_| Status::InternalServerError)?;
    }

    Ok(())
}
