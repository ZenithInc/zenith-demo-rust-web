use tracing::{event, Level};
use crate::utils;

pub fn init_config() {
    utils::config::init();
    event!(Level::INFO, "config initialized");
}