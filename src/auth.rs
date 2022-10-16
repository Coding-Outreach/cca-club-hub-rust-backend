use argon2::Argon2;
use jsonwebtoken::{
    errors::Result as JwtResult, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use password_hash::{
    self, rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
};
use serde::{Deserialize, Serialize};
use std::{ops::Deref, time::Duration};

pub fn hash_password(password: impl AsRef<[u8]>) -> password_hash::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_ref(), &salt)
        .map(|h| h.to_string())
}

pub fn verify_password(
    password: impl AsRef<[u8]>,
    password_hash: impl AsRef<str>,
) -> password_hash::Result<bool> {
    let parsed_hash = PasswordHash::new(password_hash.as_ref())?;
    Ok(Argon2::default()
        .verify_password(password.as_ref(), &parsed_hash)
        .is_ok())
}

struct Keys {
    encoding: EncodingKey,
    #[allow(unused)]
    decoding: DecodingKey,
}

lazy_static::lazy_static! {
    // TODO: use jwt_secret from config instead of env var
    static ref KEYS: Keys = {
        let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        Keys {
            encoding: EncodingKey::from_base64_secret(&secret).expect("JWT_SECRET is not valid base64"),
            decoding: DecodingKey::from_base64_secret(&secret).expect("JWT_SECRET is not valid base64"),
        }
    };
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub club_id: i32,
    pub exp: u64,
}

#[allow(unused_must_use)]
pub fn ensure_jwt_secret_is_valid() {
    KEYS.deref();
}

pub fn generate_jwt(club_id: i32, exp: Duration) -> JwtResult<String> {
    jsonwebtoken::encode(
        &Header::default(),
        &Claims {
            club_id,
            exp: jsonwebtoken::get_current_timestamp() + exp.as_secs(),
        },
        &KEYS.encoding,
    )
}

pub fn validate_jwt(token: &str) -> JwtResult<TokenData<Claims>> {
    jsonwebtoken::decode::<Claims>(token, &KEYS.decoding, &Validation::default())
}
