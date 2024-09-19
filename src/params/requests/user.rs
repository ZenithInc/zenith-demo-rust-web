use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Deserialize, Serialize, Validate)]
pub struct LoginParams {

    #[validate(length(min = 3, max = 32))]
    pub username: String,

    #[validate(length(min = 8))]
    pub password: String,

}

