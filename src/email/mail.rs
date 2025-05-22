use crate::email::models::*;
use crate::email::send_mails::send_email;
use crate::email::templetes::mail_templates::VERIFICATION_EMAIL;
use rocket::serde::json::Json;
use std::env;

pub async fn send_verification_email(
    user_data: Json<EmailVerificationRequest>,
) -> Result<(), Box<dyn std::error::Error>> {
    let email = user_data.email.clone();
    let base_url = env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8000".to_string());
    let verification_url = format!("{}/api/verify-email/{}", base_url, user_data.token);
    let placeholders = vec![("VERIFICATION_URL".to_string(), verification_url)];
    let template = VERIFICATION_EMAIL;
    send_email(&email, "Verify your email", template, &placeholders).await
}
