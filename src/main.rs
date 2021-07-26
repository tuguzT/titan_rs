#![windows_subsystem = "windows"]

use std::error::Error;

use pretty_env_logger as logger;

use titan_engine::config::Config;
use titan_engine::run;
use titan_engine::window::Event;

const APP_NAME: &str = env!("CARGO_CRATE_NAME", "library must be compiled by Cargo");
const APP_VERSION_STR: &str = env!("CARGO_PKG_VERSION", "library must be compiled by Cargo");

fn main() -> Result<(), Box<dyn Error>> {
    logger::try_init()?;

    let version = APP_VERSION_STR.parse()?;
    let config = Config::new(APP_NAME.to_string(), version);
    run(config, move |event| match event {
        Event::Created => {
            log::debug!("created");
        }
        Event::Resized(size) => {
            let size: (u32, u32) = size.into();
            log::debug!("resized with {:?}", size);
        }
        Event::Destroyed => {
            log::debug!("destroyed");
        }
    })
}
