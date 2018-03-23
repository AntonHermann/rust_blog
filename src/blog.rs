use auth::UserGuard;
use rocket_contrib::Template;
use rocket::Route;
use rocket::response::{Flash, Redirect};
use std::collections::HashMap;
use rocket::request::Form;
use diesel::prelude::*;
use models::*;
use DbConn;

/// exports this modules routes to the public (hiding the other functions)
pub fn routes() -> Vec<Route> {
    routes![
        get_post,
        create_post,
        new_post,
    ]
}

/// a blog entry as retreived from the new post form
#[derive(FromForm)]
struct NewBlogEntry {
    title: String,
    content: String,
}

/// request the blog post of given name
#[get("/<title>", rank = 1)]
fn get_post(title: String, conn: DbConn) -> Option<Template> {
    let post: Post = Post::with_title(&title).first(&*conn).ok()?;
    Some(Template::render("single_post", post))
}

/// create a new blog post
#[post("/", data = "<formdata>")]
fn create_post(formdata: Form<NewBlogEntry>, u: UserGuard, conn: DbConn)
    -> Flash<Redirect>
{
    let formdata = formdata.into_inner();
    let new_post = NewPost::new(formdata.title, formdata.content, u.id());
    match new_post.insert(&*conn) {
        Ok(post) => {
            let title = post.title;
            Flash::success(
                Redirect::to(&format!("/blog/{}", title)),
                format!("Successfully created new blog entry: {}", title)
            )
        },
        Err(_) => {
            Flash::error(Redirect::to("/blog/new"), "error creating post")
        }
    }
}

#[get("/new")]
fn new_post(_user: UserGuard) -> Template {
    let context: HashMap<String, String> = HashMap::new();
    Template::render("new_blog", context)
}
