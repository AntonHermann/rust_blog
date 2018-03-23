#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate file_server_lib;
extern crate rocket;
// #[macro_use] extern crate rocket_contrib;
extern crate rocket_contrib;
extern crate serde;
extern crate serde_json;
// #[macro_use]
// extern crate serde_derive;
extern crate itertools;
extern crate uuid;
// #[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate chrono;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate tera;

use file_server_lib::*;
use file_server_lib::models::*;
use rocket_contrib::Template;
// use std::path::{Path, PathBuf};
// use rocket::response::NamedFile;
use rocket::Rocket;
use tera::Context;
use diesel::prelude::*;

#[get("/")]
fn index(conn: DbConn) -> Template {
    use schema::posts::dsl::*;
    use schema::users::dsl::*;

    let mut context = Context::new();

    let post_list = posts.load::<Post>(&*conn).expect("error loading posts");
    let user_list = users.load::<User>(&*conn).expect("error loading posts");

    context.add("posts", &post_list);
    context.add("users", &user_list);

    Template::render("test", context)
}


/// build the rocket
fn rocket() -> Rocket {
    rocket::ignite()
        .attach(Template::fairing())
        .manage(init_pool())
        .mount("/", auth::routes())
        .mount("/blog", blog::routes())
        .mount("/", routes![index])
        .mount("/", static_files::routes())
        .catch(static_files::err())
}

fn main() {
    rocket().launch();
}
