use blocks::*;
use std::process::Command;
use std::thread;
// use std::time::Duration;
use std::sync::{Arc};
use rocket::State;
use rocket_contrib::Json;
use std::io::Read;
use std::error::Error;
use std::fs::OpenOptions;
use inotify::{EventMask, Inotify, WatchMask};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

// #[derive(Debug, Serialize, PartialEq, Clone)]
pub struct BacklightBlock {
    shared: Shared,
}

type Shared = Arc<AtomicUsize>;

impl Block for BacklightBlock {
    type Managed = Shared;
    type OutputFormat = u32;

    fn new() -> Result<Self> {
        let device_path = get_device_path()?;
        let max_brightness = fetch(&device_path, Some("max_brightness"), 0)?;
        let current_brightness = fetch(&device_path, None, max_brightness)?;
        let shared = Arc::new(AtomicUsize::new(current_brightness as usize));
        let sh_cloned = shared.clone();
        let dp_cloned = device_path.clone();
        thread::spawn(move || {
            observe_system_backlight(sh_cloned, dp_cloned, max_brightness);
        });
        Ok(BacklightBlock { shared })
    }
    fn get_managed(&self) -> Self::Managed {
        self.shared.clone()
    }
    fn get_routes(&self) -> Vec<rocket::Route> {
        routes![get_brightness, set_brightness]
    }
    fn get_info(i: State<Self::Managed>) -> Result<Self::OutputFormat> {
        let r = i.inner().load(Ordering::Relaxed);
        Ok(r as u32)
    }
}

pub fn fetch(device_path: &Path, attr: Option<&str>, max: u32) -> Result<u32> {
    let mut file = OpenOptions::new().read(true)
        .open(device_path.join(attr.unwrap_or("brightness")))
        .map_err(|e| e.description().to_owned())?;
    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(|e| e.description().to_owned())?;
    content.pop(); // remove trailing newline
    let backlight = content.parse::<u32>()
        .map_err(|e| e.description().to_owned())?;
    if attr.is_none() { //some is only the case on first init when max is 0
        Ok(backlight * 100 / max)
    } else {
        Ok(backlight)
    }
}

fn get_device_path() -> Result<PathBuf> {
    let devices = Path::new("/sys/class/backlight").read_dir()
        .map_err(|e| e.description().to_owned())?;
    if let Some(Ok(first_device)) = devices.take(1).next() {
        Ok(first_device.path())
    } else {
        Err("no backlight device found".to_string())
    }
}

fn observe_system_backlight(sh: Shared, dev_path: PathBuf, max: u32) {
    let mut notify = Inotify::init().expect("Failed to start inotify");
    let file = dev_path.join("brightness");
    notify.add_watch(file, WatchMask::MODIFY)
        .expect("Failed to watch brightness file");
    let mut buffer = [0; 1024];
    loop {
        let mut events = notify.read_events_blocking(&mut buffer)
            .expect("Error while reading inotify events");
        if events.any(|ev| ev.mask.contains(EventMask::MODIFY)) {
            let new_brightness = fetch(&dev_path, None, max)
                .expect("Error fetching new backlight brightness");
            sh.store(new_brightness as usize, Ordering::SeqCst);
        }
    }
}

#[get("/")]
fn get_brightness(shared: State<Shared>) -> Result<Json<u32>> {
    let r = shared.inner().load(Ordering::Relaxed);
    Ok(Json(r as u32))
}

#[get("/<brightness>")]
fn set_brightness(brightness: u32, shared: State<Shared>) -> Result<Json<u32>> {
    Command::new("xbacklight")
        .args(&["-set", &*brightness.to_string()])
        .output().map_err(|e| e.description().to_owned())?;
    get_brightness(shared)
}
