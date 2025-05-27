use crate::email::mail::send_verification_email;
use crate::email::models::EmailVerificationRequest;
use crate::user_mutate::models::NewCurrentUser;
use crate::user_mutate::utils_functions::create_new_user;
use crate::utils::db::Connection;
use rocket::http::Status;
use rocket::serde::json::Json;
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

// In-memory storage for verification tokens
lazy_static::lazy_static! {
    static ref VERIFICATION_TOKENS: Mutex<HashMap<String, NewCurrentUser>> = Mutex::new(HashMap::new());
}

/// Send verification email to the user
#[post("/register", data = "<user_data>")]
pub async fn user_registor(user_data: Json<NewCurrentUser>) -> Result<Status, Status> {
    let token = Uuid::new_v4().to_string();
    let user_data = user_data.into_inner();

    // Create email verification request
    let email_request = EmailVerificationRequest {
        name: user_data.name.clone(),
        email: user_data.email.clone(),
        password: user_data.password.clone(),
        token: token.clone(),
    };

    // Send verification email
    if let Err(e) = send_verification_email(Json(email_request)).await {
        println!("Error sending verification email: {:?}", e);
        return Err(Status::InternalServerError);
    }

    // Store user data with token
    VERIFICATION_TOKENS
        .lock()
        .unwrap()
        .insert(token.clone(), user_data);

    Ok(Status::Ok)
}

/// Verify email and create user
#[get("/verify-email/<token>")]
pub async fn user_verify_email(token: &str, conn: Connection) -> Result<String, Status> {
    // Get the user data associated with the token
    let user_data = {
        let mut tokens = VERIFICATION_TOKENS.lock().unwrap();
        tokens.remove(&token.to_string()).ok_or(Status::NotFound)?
    };

    // Create the user in the database
    match create_new_user(conn, user_data).await {
        Ok(_) => Ok(format!(
            r#"
            <html>
                <body>
                    <h2>Email Verified</h2>
                    <p>Your email has been verified successfully. You can now login to your account.</p>
                </body>
            </html>
            "#
        )),
        Err(status) => Err(status),
    }
}
