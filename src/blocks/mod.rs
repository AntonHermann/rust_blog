mod volume;
mod backlight;
mod wifi;

pub use self::volume::*;
pub use self::backlight::*;
pub use self::wifi::*;

use rocket::{self, State};
use super::Result;

pub trait Block {
    type Managed;
    type OutputFormat;

    fn new() -> Result<Self> where Self: Sized;
    fn get_managed(&self) -> Self::Managed;
    fn get_routes(&self) -> Vec<rocket::Route>;
    fn get_info(i: State<Self::Managed>) -> Result<Self::OutputFormat>
        where Self::Managed: Sync + Send;
}
