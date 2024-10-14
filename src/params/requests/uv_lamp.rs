use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct TurnParams {
    #[validate(range(min = 100_000, max = 999_999))]
    pub message_id: i32,

    #[validate(length(min = 12, max = 18))]
    pub device_number: String,

    pub status: bool,
}
