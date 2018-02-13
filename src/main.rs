#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
#[macro_use] extern crate rocket_contrib;
extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate inotify;

mod blocks;

pub use blocks::*;

use rocket_contrib::{Json, Value};
use rocket::State;

pub type Result<T> = std::result::Result<T, String>;

#[derive(Serialize)]
struct AllBlocks {
    pub volume: <VolumeBlock as Block>::OutputFormat,
    pub brightness: <BacklightBlock as Block>::OutputFormat,
}

#[get("/")]
fn get(vol: State<<VolumeBlock    as Block>::Managed>,
       bri: State<<BacklightBlock as Block>::Managed>) -> Result<Json<AllBlocks>> {
    let volume = VolumeBlock::get_info(vol)?;
    let brightness = BacklightBlock::get_info(bri)?;

    Ok(Json(AllBlocks { volume, brightness } ))
}

#[error(404)]
fn not_found() -> Json<Value> {
    Json(json!({
        "status": "error",
        "reason": "Resource was not found."
    }))
}

fn main() {
    let vol: VolumeBlock = VolumeBlock::new()
        .expect("Couldn't load VolumeBlock");
    let bri: BacklightBlock = BacklightBlock::new()
        .expect("Couldn't load BacklightBlock");
    rocket::ignite()
        .mount("/vol", vol.get_routes())
        .mount("/bri", bri.get_routes())
        .mount("/", routes![get])
        .catch(errors![not_found])
        .manage(vol.get_managed())
        .manage(bri.get_managed())
        .launch();
}
