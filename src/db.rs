use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use rocket::request::{self, FromRequest};
use rocket::{outcome::Outcome, Request};
use std::env;

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type DbConn = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub fn run_migrations(connection: &mut PgConnection) {
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to run database migrations");
    // list all tables
    let tables = diesel::sql_query(
        "SELECT table_name FROM information_schema.tables WHERE table_schema='public'",
    )
    .execute(connection)
    .expect("Failed to list tables");
    println!("Tables: {:?}", tables);
}

pub fn establish_connection() -> Pool {
    println!("DATABASE_URL: {}", std::env::var("DATABASE_URL").unwrap());
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);

    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool");

    run_migrations(&mut pool.get().expect("Failed to get connection for migrations"));

    pool
}

pub struct Connection(pub DbConn);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Connection {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let pool = request.rocket().state::<Pool>().unwrap();
        match pool.get() {
            Ok(conn) => Outcome::Success(Connection(conn)),
            Err(_) => Outcome::Error((rocket::http::Status::ServiceUnavailable, ())),
        }
    }
}
