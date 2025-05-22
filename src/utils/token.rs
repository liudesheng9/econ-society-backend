use uuid::Uuid;

pub fn generate_token() -> String {
    let token = Uuid::new_v4().to_string();
    token
}
