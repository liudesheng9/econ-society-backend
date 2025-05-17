use crate::user_login::api_functions::create_new_user;
use crate::user_login::models::NewCurrentUser;
use crate::utils::db::Connection;
use lettre::message::{header::ContentType, Message};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Tokio1Executor};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use std::collections::HashMap;
use std::env;
use std::sync::Mutex;
use uuid::Uuid;

// In-memory storage for verification tokens
lazy_static::lazy_static! {
    static ref VERIFICATION_TOKENS: Mutex<HashMap<String, NewCurrentUser>> = Mutex::new(HashMap::new());
}

#[derive(serde::Deserialize)]
pub struct EmailVerificationRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(serde::Serialize)]
pub struct EmailVerificationResponse {
    pub message: String,
}

/// Send verification email to the user
#[post("/register", data = "<user_data>")]
pub async fn send_verification_email(
    user_data: Json<EmailVerificationRequest>,
) -> Result<Json<EmailVerificationResponse>, Status> {
    let token = Uuid::new_v4().to_string();

    // Store the user data with the token for later verification
    let new_user = NewCurrentUser {
        name: user_data.name.clone(),
        email: user_data.email.clone(),
        password: user_data.password.clone(),
    };

    // Save to in-memory store
    VERIFICATION_TOKENS
        .lock()
        .unwrap()
        .insert(token.clone(), new_user);

    // Get email settings from environment variables
    let smtp_server = env::var("SMTP_SERVER").unwrap_or_else(|_| "smtp.gmail.com".to_string());
    let smtp_port = env::var("SMTP_PORT").unwrap_or_else(|_| "587".to_string());
    let smtp_username = env::var("SMTP_USERNAME").expect("SMTP_USERNAME must be set");
    let smtp_password = env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set");
    let from_email = env::var("FROM_EMAIL").unwrap_or_else(|_| smtp_username.clone());

    println!("SMTP Server: {}", smtp_server);
    println!("SMTP Port: {}", smtp_port);
    println!("SMTP Username: {}", smtp_username);
    println!("SMTP Password: {}", smtp_password);
    println!("From Email: {}", from_email);
    println!("User Data: {}", user_data.email);

    // Create the verification URL
    let base_url = env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8000".to_string());
    let verification_url = format!("{}/api/verify-email/{}", base_url, token);

    // Compose the email
    let email = Message::builder()
        .from(from_email.parse().unwrap())
        .to(user_data.email.parse().unwrap())
        .subject("Email Verification")
        .header(ContentType::TEXT_HTML)
        .body(format!(
            r#"
            <html>
                <body>
                    <h2>Verify Your Email</h2>
                    <p>Thank you for registering! Please click the link below to verify your email address:</p>
                    <p><a href="{}">Verify Email</a></p>
                    <p>If you didn't request this, please ignore this email.</p>
                </body>
            </html>
            "#,
            verification_url
        ))
        .map_err(|_| Status::InternalServerError)?;

    // Set up the SMTP transport
    let creds = Credentials::new(smtp_username, smtp_password);
    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp_server)
        .map_err(|_| Status::InternalServerError)?
        .credentials(creds)
        .port(smtp_port.parse::<u16>().unwrap_or(587))
        .build();

    // Send the email
    match mailer.send(email).await {
        Ok(_) => Ok(Json(EmailVerificationResponse {
            message: "Verification email sent. Please check your inbox.".to_string(),
        })),
        Err(e) => {
            println!("Error sending email: {:?}", e);
            Err(Status::InternalServerError)
        }
    }
}

/// Verify email and create user
#[get("/verify-email/<token>")]
pub async fn verify_email(token: String, conn: Connection) -> Result<String, Status> {
    // Get the user data associated with the token
    let user_data = {
        let mut tokens = VERIFICATION_TOKENS.lock().unwrap();
        tokens.remove(&token).ok_or(Status::NotFound)?
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
