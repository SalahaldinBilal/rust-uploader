use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use jwt::VerifyWithKey;
use sha2::Sha256;
use std::collections::BTreeMap;

pub fn get_env_value(name: &str) -> String {
    std::env::var(name).expect(&format!("Env variable {} should exist", name))
}

pub fn get_file_extension(filename: &str) -> String {
    let split_name: Vec<&str> = filename.split(".").collect();

    if let Some(extension) = split_name.last() {
        return extension.to_string();
    } else {
        return "".to_string();
    }
}

pub fn create_jwt_token(secret: &str, claims: &BTreeMap<&str, &str>) -> Result<String, jwt::Error> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(&secret.as_bytes()).unwrap();
    claims.sign_with_key(&key)
}

pub fn verify_jwt_token(secret: &str, token: &str) -> Result<BTreeMap<String, String>, jwt::Error> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(&secret.as_bytes()).unwrap();
    let claims: Result<BTreeMap<String, String>, jwt::Error> = token.verify_with_key(&key);
    claims
}
