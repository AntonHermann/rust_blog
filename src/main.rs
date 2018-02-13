#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
#[macro_use] extern crate rocket_contrib;
extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;

use std::process::{Command, Stdio};
use rocket_contrib::{Json, Value};
use std::error::Error;
use std::sync::{RwLock, Arc};
use rocket::State;
use std::io::Read;
use std::thread;
use std::time::Duration;

#[derive(Debug, Serialize, PartialEq, Clone)]
struct VolumeInfo {
    pub vol: u32,
    pub muted: bool,
}

impl VolumeInfo {
    pub fn fetch() -> VolumeInfo {
        let output: String = Command::new("sh")
            .args(&["-c", "amixer get Master"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
            .expect("error getting volume info");
        let last = output.lines().last().expect("coulnd't get left channel");
        let mut els = last.split_whitespace().filter(|x| x.starts_with('['))
            .map(|s| s.trim_matches(FILTER_PATTERN));
        let vol = els.next().expect("coulnd't read volume").parse::<u32>()
            .expect("failed parsing volume");
        let muted = els.next().expect("couldn't get muted state") == "off";
        VolumeInfo { vol, muted }
    }
}

const FILTER_PATTERN: &[char] = &['[', ']', '%'];

type Result<T> = std::result::Result<T, String>;
type CurrVolInfo = Arc<RwLock<VolumeInfo>>;

fn observe_system_sound(vol_info: CurrVolInfo) {
    let mut monitor = Command::new("sh")
        .args(&["-c", "stdbuf -oL alsactl monitor"])
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start alsactl monitor")
        .stdout
        .expect("Failed to pipe alsactl monitor output");
    let mut buffer = [0; 1024];
    loop {
        if let Ok(_) = monitor.read(&mut buffer) {
            let new_vol_info = VolumeInfo::fetch();
            if *vol_info.read().unwrap() != new_vol_info {
                let mut w = vol_info.write().unwrap();
                *w = new_vol_info;
            }
        }
        thread::sleep(Duration::new(0,250_000_000))
    }
}

#[get("/")]
fn get_volume(vol_info: State<CurrVolInfo>) -> Result<Json<VolumeInfo>> {
    let r = &*vol_info.inner().read().map_err(|e| e.description().to_owned())?;
    Ok(Json(r.clone()))
}

#[get("/<vol>")]
fn set_volume(vol: u32, vol_info: State<CurrVolInfo>) -> Result<Json<VolumeInfo>> {
    Command::new("sh")
        .args(&["-c", format!("amixer set Master {}%", vol).as_str()])
        .output().map_err(|e| e.description().to_owned())?;
    {
        let mut w = vol_info.inner().write()
            .map_err(|e| e.description().to_owned())?;
        w.vol = vol;
    }
    get_volume(vol_info)
}

#[get("/<muted>", rank=2)]
fn set_mute(muted: bool, vol_info: State<CurrVolInfo>) -> Result<Json<VolumeInfo>> {
    let action = if muted { "mute" } else { "unmute" };
    Command::new("sh")
        .args(&["-c", format!("amixer set Master {}", action).as_str()])
        .output().map_err(|e| e.description().to_owned())?;
    {
        let mut w = vol_info.inner().write()
            .map_err(|e| e.description().to_owned())?;
        w.muted = muted;
    }
    get_volume(vol_info)
}

#[error(404)]
fn not_found() -> Json<Value> {
    Json(json!({
        "status": "error",
        "reason": "Resource was not found."
    }))
}

fn main() {
    let vol_info = Arc::new(RwLock::new(VolumeInfo::fetch()));
    let cloned = vol_info.clone();
    thread::spawn(move || {
        observe_system_sound(cloned);
    });
    rocket::ignite()
        .mount("/", routes![get_volume, set_volume, set_mute])
        .catch(errors![not_found])
        .manage(vol_info)
        .launch();
}
