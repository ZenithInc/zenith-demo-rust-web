use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginSuccess {
    pub token: String,
}

impl From<String> for LoginSuccess {
    fn from(token: String) -> Self {
        LoginSuccess { token }
    }
}
