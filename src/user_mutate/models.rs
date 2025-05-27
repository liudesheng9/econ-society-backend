use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Insertable, Serialize, Selectable, Debug)]
#[diesel(table_name = crate::schema::current_users)]
pub struct CurrentUser {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub password: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Deserialize)]
pub struct NewCurrentUser {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, Serialize)]
pub struct UserReduced {
    pub id: i32,
    pub name: String,
}

impl From<CurrentUser> for UserReduced {
    fn from(user: CurrentUser) -> Self {
        UserReduced {
            id: user.id,
            name: user.name,
        }
    }
}

#[derive(Deserialize)]
pub struct LoginData {
    pub email: String,
    pub password: String,
}

impl Clone for LoginData {
    fn clone(&self) -> Self {
        LoginData {
            email: self.email.clone(),
            password: self.password.clone(),
        }
    }
}
#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    pub hash: String,
}
