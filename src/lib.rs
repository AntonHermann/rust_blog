#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

// database
#[macro_use] extern crate diesel;
extern crate r2d2;
extern crate r2d2_diesel;

// rocket
extern crate rocket;
// #[macro_use] extern crate rocket_contrib;
extern crate rocket_contrib;

// serialisation
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

extern crate itertools;
extern crate uuid;
extern crate dotenv;
extern crate chrono;
extern crate bcrypt;
#[macro_use]
extern crate error_chain;

pub mod schema;
pub mod models;
pub mod auth;
pub mod blog;
pub mod static_files;
pub mod errors;

use std::ops::Deref;
use std::env;
use dotenv::dotenv;
use diesel::PgConnection;
use r2d2::{Pool, PooledConnection};
use r2d2_diesel::ConnectionManager;
use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Request, State, Outcome};
pub use errors::*;

pub fn init_pool() -> Pool<ConnectionManager<PgConnection>> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set!");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::new(manager).expect("Failed to create db pool!")
}

pub struct DbConn(pub PooledConnection<ConnectionManager<PgConnection>>);

impl<'a, 'r> FromRequest<'a, 'r> for DbConn {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<DbConn, ()> {
        let pool = request
            .guard::<State<Pool<ConnectionManager<PgConnection>>>>()?;
        match pool.get() {
            Ok(conn) => Outcome::Success(DbConn(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ())),
        }
    }
}

impl Deref for DbConn {
    type Target = PgConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
