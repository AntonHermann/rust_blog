#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
#[macro_use] extern crate rocket_contrib;
extern crate diesel;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate db;
extern crate uuid;
extern crate chrono;
extern crate serde;
#[macro_use] extern crate serde_derive;

mod database;

use database::DbConn;
use db::models::Post;
use rocket_contrib::{Json, UUID, Value};
use diesel::prelude::*;
use uuid::Uuid;
use chrono::prelude::*;

#[get("/")]
fn posts(conn: DbConn) -> QueryResult<Json<Vec<Post>>> {
    use db::schema::posts::dsl::*;
    posts.load::<Post>(&*conn).map(|post| Json(post))
}

#[get("/<id>")]
fn get_post(conn: DbConn, id: UUID) -> QueryResult<Json<Post>> {
    use db::schema::posts::dsl::*;
    posts.filter(uuid.eq(id.into_inner()))
        .first::<Post>(&*conn)
        .map(|post| Json(post))
}

#[delete("/<id>")]
fn delete_post(conn: DbConn, id: UUID) -> QueryResult<Json<usize>> {
    use db::schema::posts;
    diesel::delete(posts::table.filter(posts::uuid.eq(id.into_inner())))
        .execute(&*conn).map(|c| Json(c))
}


#[derive(Clone, Debug, Deserialize)]
pub struct NewPost {
    pub title:  String,
    pub body:   String,
    pub author: String,
}

#[post("/", data="<post>")]
fn post_post(conn: DbConn, post: Json<NewPost>) -> Option<Json<Uuid>> {
    let NewPost {title, body, author} = post.0;
    new_post(conn, title, body, author)
}

#[get("/new/<title>/<body>/<author>")]
fn new_post(conn: DbConn, title: String, body: String, author: String)
    -> Option<Json<Uuid>>
{
    use db::schema::posts;

    let datetime: NaiveDateTime = Utc::now().naive_utc();
    let uuid: Uuid = Uuid::new_v4();
    let newpost = Post { title, body, author, datetime, uuid };
    let res = diesel::insert_into(posts::table).values(&newpost).execute(&*conn);
    match res {
        Ok(1) => Some(Json(uuid)),
        Ok(_) => None,
        Err(_) => None,
    }
}

#[error(404)]
fn not_found() -> Json<Value> {
    Json(json!({
        "status": "error",
        "reason": "Resource was not found."
    }))
}

fn main() {
    let post_routes = routes![posts, get_post, post_post, delete_post, new_post];
    rocket::ignite()
        .mount("/post", post_routes)
        .catch(errors![not_found])
        .manage(db::init_pool())
        .launch();
}
