use rand_core::OsRng;
use argon2::{
    Argon2, 
    password_hash::SaltString,
    PasswordHash,
    PasswordHasher,
    PasswordVerifier,
};

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let config = Argon2::default();
    let hash = config.hash_password(password.as_bytes(), &salt)?.to_string();
    Ok(hash)
}

pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, anyhow::Error> {
    let config = Argon2::default();
    let password_hash = PasswordHash::new(&password_hash).map_err(|e| anyhow::anyhow!(e))?;
    let pass = config.verify_password(password.as_bytes(), &password_hash).is_ok();
    Ok(pass)
}

#[cfg(test)]
mod test {
    use super::{hash_password, verify_password};

    #[test]
    fn test_hash_password() {
        let password = "password";
        let hash = hash_password(password).unwrap();
        assert_ne!(password, hash);
    }

    #[test]
    fn test_verify_password() {
        let password = "password";
        let hash = hash_password(password).unwrap();
        let pass = verify_password(password, &hash).unwrap();
        assert_eq!(pass, true);
    }
}
