use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Insertable, Serialize, Selectable)]
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
