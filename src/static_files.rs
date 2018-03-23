use rocket::{Route, Catcher};
use rocket::response::NamedFile;
use std::path::{Path, PathBuf};
use rocket;

pub fn routes() -> Vec<Route> {
    routes![files]
}

pub fn err() -> Vec<Catcher> {
    errors![not_found, blocked]
    // errors![]
}

/// show static file
#[get("/static/<file..>", rank = 3)]
fn files(file: PathBuf) -> Option<NamedFile> {
    let path = Path::new("static/").join(file);
    NamedFile::open(path).ok()
}

/// "404 - Not Found" error route
/// has a fallback to a static string if even the 404 error page couldn't
/// be loaded to avoid recursive error page loads
#[error(404)]
fn not_found() -> Result<NamedFile, String> {
    let path = Path::new("static/404_Not_Found.html");
    if path.exists() {
        if let Ok(nf) = NamedFile::open(path) {
            return Ok(nf);
        }
    }
    eprintln!("404 file wasn't found");
    Err("404 - Not found".to_string())
}

/// "401 - Unauthorized" error route
/// has a fallback to a static string if even the 401 error page couldn't load
#[error(401)]
fn blocked() -> Result<NamedFile, String> {
    let path = Path::new("static/401_Unauthorized.html");
    if path.exists() {
        if let Ok(nf) = NamedFile::open(path) {
            return Ok(nf);
        }
    }
    eprintln!("401 access denied");
    Err("401 - Access Denied".to_string())
}
