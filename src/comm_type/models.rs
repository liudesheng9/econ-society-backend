use serde::Deserialize;

/// A wrapper struct containing a type T and a user token
#[derive(Deserialize)]
pub struct CommInWrapper<T> {
    pub data: T,
    pub user_hash: String,
}

impl<T> CommInWrapper<T> {
    pub fn get_user_hash(&self) -> &str {
        &self.user_hash
    }

    pub fn get_data(&self) -> &T {
        &self.data
    }
}
