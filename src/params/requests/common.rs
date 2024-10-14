use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Deserialize, Serialize, Validate)]
pub struct IdParams {
    #[validate(range(min = 1))]
    pub id: i32,
}
