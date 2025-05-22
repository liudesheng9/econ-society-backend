use serde::{Deserialize, Serialize};

#[derive(serde::Deserialize)]
pub struct EmailVerificationRequest {
    pub name: String,
    pub email: String,
    pub password: String,
    pub token: String,
}

#[derive(serde::Serialize)]
pub struct EmailVerificationResponse {
    pub message: String,
}
