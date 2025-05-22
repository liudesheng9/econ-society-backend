use serde::Serialize;

/// A wrapper struct containing a type T and a user token
#[derive(Serialize)]
pub struct CommInWrapper<T>
where
    T: Serialize,
{
    pub data: T,
    pub user_token: String,
}
