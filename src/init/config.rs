use crate::utils;
use tracing::{event, Level};

pub fn init_config() {
    utils::config::init();
    event!(Level::INFO, "config initialized");
}
