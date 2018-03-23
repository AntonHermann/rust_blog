// use rocket::outcome::{IntoOutcome, Outcome};
use rocket::outcome::Outcome;
use rocket::request::{Form, FromRequest, FlashMessage, Request};
use rocket::request::Outcome as rOutcome;
use rocket::response::{Flash, Redirect};
use rocket::http::{Cookies, Cookie};
use rocket::response::status;
use rocket::http::Status;
use rocket::Route;
use serde_json;
use std::collections::HashMap;
use rocket_contrib::Template;
use uuid::Uuid;
use models::User;
use DbConn;
use diesel::prelude::*;
use bcrypt::verify;

/// form data of login form
#[derive(FromForm)]
struct UserLoginData {
    name: String,
    pwd: String,
}

#[derive(Serialize, Deserialize)]
pub struct UserGuard {
    id: Uuid,
    name: String,
}

/// allows User to be used as request guard
/// (accepting routes only if a user is logged in)
impl<'a, 'r> FromRequest<'a, 'r> for UserGuard {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> rOutcome<UserGuard, ()> {
        match request.cookies()
            .get_private("user")
            .and_then(|cookie| serde_json::from_str(cookie.value()).ok())
        {
            Some(user) => Outcome::Success(user),
            None => Outcome::Failure((Status::Unauthorized, ())),
        }
    }
}
impl UserGuard {
    pub fn id(&self) -> Uuid {
        self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// exports this modules routes to the public (hiding the other functions)
pub fn routes() -> Vec<Route> {
    routes![login, logout, login_page, user_index]
}

/// handles a login form submit
#[post("/login", data = "<logindata>")]
fn login(mut cookies: Cookies, logindata: Form<UserLoginData>, conn: DbConn)
    -> Result<Flash<Redirect>, status::Custom<&'static str>>
{
    let failure = Err(status::Custom(
        Status::Unauthorized,
        "Invalid username/password"
    ));
    let userdata: UserLoginData = logindata.into_inner();
    let user: User = match User::with_name(&userdata.name).first(&*conn) {
        Ok(user) => user,
        Err(_) => return failure,
    };
    match verify(&userdata.pwd, &user.password) {
        Ok(true) => { // valid password
            let u = UserGuard { id: user.id, name: user.name };
            cookies.add_private(
                Cookie::new("user", serde_json::to_string(&u).unwrap()));
            Ok(Flash::success(Redirect::to("/"), "Successfully logged in"))
        },
        _ => failure,
    }
}

/// handles user logout
/// (doing effectively nothing if there is no user logged in)
#[post("/logout")]
fn logout(mut cookies: Cookies) -> Flash<Redirect> {
    cookies.remove_private(Cookie::named("user"));
    Flash::success(Redirect::to("/login"), "Successfully logged out.")
}

/// login form (optionally showing a status message,
/// i.e. if the recent login attempt failed)
/// or a redirect to root if already logged in
#[get("/login", rank = 2)]
fn login_page(flash: Option<FlashMessage>, user: Option<UserGuard>)
    -> Result<Template, Redirect>
{
    match user {
        Some(_) => Err(Redirect::to("/")),
        None => {
            let mut context = HashMap::new();
            if let Some(ref msg) = flash {
                context.insert("flash", msg.msg());
            }
            Ok(Template::render("login", &context))
        },
    }
}

/// temporary route, shows a different index page if a user is logged in
#[get("/", rank=0)]
fn user_index(user: UserGuard) -> Template {
    let mut context = HashMap::new();
    context.insert("user_id", user.id());
    Template::render("index", &context)
}
