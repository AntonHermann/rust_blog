use blocks::*;
use std::process::Command;
use std::thread;
// use std::time::Duration;
use std::sync::{Arc, RwLock};
use rocket::State;
use rocket_contrib::Json;
use std::io::Read;
use std::error::Error;
use std::fs::OpenOptions;
use inotify::{EventMask, Inotify, WatchMask};
use std::path::{Path};

static WIFI:     &'static str = "wlp3s0";
static DEV_PATH: &'static str = "/sys/class/net/wlp3s0";

pub struct WifiBlock {
    shared: Shared,
}

type Ip = String;
type Shared = Arc<RwLock<Option<WifiData>>>;

#[derive(Debug, Serialize, PartialEq, Clone)]
pub struct WifiData {
    ssid: String,
    ip: Ip,
}

impl Block for WifiBlock {
    type Managed = Shared;
    type OutputFormat = Option<WifiData>;

    fn new() -> Result<Self> {
        let status = if is_up()? {
            let ssid = get_ssid()?;
            let ip = get_ip()?;
            Some(WifiData {ssid, ip})
        } else {
            None
        };
        let shared = Arc::new(RwLock::new(status));
        let sh_cloned = shared.clone();
        thread::spawn(move || {
            observe_wifi(sh_cloned);
        });
        Ok(WifiBlock { shared })
    }
    fn get_managed(&self) -> Self::Managed {
        self.shared.clone()
    }
    fn get_routes(&self) -> Vec<rocket::Route> {
        routes![get_connection]
    }
    fn get_info(i: State<Self::Managed>) -> Result<Self::OutputFormat> {
        let r = &*i.inner().read().map_err(|e| e.description().to_owned())?;
        // r.clone().ok_or("Error getting wifi state".to_string())
        Ok(r.clone())
    }
}

fn is_up() -> Result<bool> {
    let opstate_file = Path::new(DEV_PATH).join("operstate");
    if !opstate_file.exists() {
        Ok(false)
    } else {
        let mut f = OpenOptions::new().read(true).open(&opstate_file)
            .map_err(|e| e.description().to_owned())?;
        let mut content = String::new();
        f.read_to_string(&mut content).map_err(|e| e.description().to_owned())?;
        content.pop(); // remove trailing newline
        Ok("up" == content)
    }
}

fn get_ssid() -> Result<String> {
    let mut iw_out = Command::new("sh")
        .args(&["-c", &format!("iw dev {} link | grep \"^\\sSSID:\" {}",
                               WIFI, "| sed \"s/^\\sSSID:\\s//g\"")])
        .output().map_err(|e| e.description().to_owned())?.stdout;
    if iw_out.len() == 0 {
        Ok("".to_string())
    } else {
        iw_out.pop(); // remove trailing newline
        Ok(String::from_utf8(iw_out).map_err(|e| e.description().to_owned())?)
    }
}

fn get_ip() -> Result<Ip> {
    let ifconfig_out = Command::new("ip")
        .args(&["-f", "inet", "-o", "address"])
        .output().map_err(|e| e.description().to_owned())?.stdout;
    let out_str: String = String::from_utf8(ifconfig_out)
        .map_err(|e| e.description().to_owned())?;
    let mut lines = out_str.lines().map(str::split_whitespace);
    let line = lines.find(|l| l.clone().nth(1) == Some(WIFI));
    let ip = line.and_then(|mut el| el.nth(3)).map(String::from);
    ip.ok_or("Error parsing wifi adress".to_string())
}

fn observe_wifi(sh: Shared) {
    let mut notify = Inotify::init().expect("Failed to start inotify");
    let file = Path::new(DEV_PATH).join("operstate");
    notify.add_watch(file, WatchMask::MODIFY)
        .expect("Failed to watch wifi connection file");
    let mut buffer = [0; 1024];
    loop {
        let mut events = notify.read_events_blocking(&mut buffer)
            .expect("Error while reading inotify events");
        if events.any(|ev| ev.mask.contains(EventMask::MODIFY)) {
            let new_status = is_up().expect("Error getting new wifi status");
            let mut w = sh.write().expect("Error get shared handle");
            *w = if new_status {
                let ssid = get_ssid().expect("Error getting SSID");
                let ip = get_ip().expect("Error getting IP");
                Some(WifiData { ssid, ip })
            } else { None };
        }
    }
}

#[get("/")]
fn get_connection(shared: State<Shared>) -> Result<Json<WifiData>> {
    let r = &*shared.inner().read().map_err(|e| e.description().to_owned())?;
    Ok(Json(r.clone().ok_or("couldn't retreive shared data")?))
}
