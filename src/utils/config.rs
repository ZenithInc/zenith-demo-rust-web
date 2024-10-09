use std::sync::Once;
use dotenv::dotenv;

static INIT: Once = Once::new();

pub fn init() {
    INIT.call_once(|| {
        dotenv().ok();
    });
}