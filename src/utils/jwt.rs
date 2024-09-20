use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use jsonwebtoken::{Header, encode, EncodingKey};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

pub fn create_token(sub: &String, secret: &str) -> Result<String, anyhow::Error> {
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("valid timestamp")
        .as_secs() + 7 * 24 * 60 * 60;

    let claims = Claims {
        sub: sub.clone(),
        exp: expiration as usize,
    };

    let result = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()));

    match result {
        Ok(token) => Ok(token),
        Err(e) => Err(anyhow::anyhow!(e.to_string())),
    }
}

#[cfg(test)]
mod test {

    use super::create_token;

    #[test]
    fn test_create_token() {
        let sub = "test".to_string();
        let secret = "secret";
        let token = create_token(&sub, secret).unwrap();
        dbg!(&token);
        assert_ne!(token, "");
    }
}
