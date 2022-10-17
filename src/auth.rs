use crate::error::{AppError, AppResult, ResponseStatusError};
use argon2::Argon2;
use axum::{
    async_trait,
    extract::{FromRequest, RequestParts},
    headers::{authorization::Bearer, Authorization},
    http::StatusCode,
    TypedHeader,
};
use jsonwebtoken::{errors::Result as JwtResult, DecodingKey, EncodingKey};
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
#[serde(rename_all = "camelCase")]
struct Claims {
    club_id: i32,
    exp: u64,
}

#[allow(unused_must_use)]
pub fn ensure_jwt_secret_is_valid() {
    KEYS.deref();
}

pub fn generate_jwt(club_id: i32, exp: Duration) -> JwtResult<String> {
    jsonwebtoken::encode(
        &Default::default(),
        &Claims {
            club_id,
            exp: jsonwebtoken::get_current_timestamp() + exp.as_secs(),
        },
        &KEYS.encoding,
    )
}

#[derive(Debug, Clone)]
pub struct Auth(Result<i32, ResponseStatusError>);

#[async_trait]
impl<B: Send> FromRequest<B> for Auth {
    type Rejection = ();

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        async fn inner<B: Send>(req: &mut RequestParts<B>) -> Result<i32, ResponseStatusError> {
            let TypedHeader(Authorization(bearer)) = req
                .extract::<TypedHeader<Authorization<Bearer>>>()
                .await
                .map_err(|_| (StatusCode::BAD_REQUEST, "missing credentials"))?;
            let claims =
                jsonwebtoken::decode::<Claims>(bearer.token(), &KEYS.decoding, &Default::default())
                    .map_err(|_| (StatusCode::BAD_REQUEST, "invalid token"))?
                    .claims;

            if claims.exp < jsonwebtoken::get_current_timestamp() {
                Err((StatusCode::UNAUTHORIZED, "token expired"))?
            } else {
                Ok(claims.club_id)
            }
        }

        Ok(Self(inner(req).await))
    }
}

impl Auth {
    pub fn is_authorized(self, club_id: i32) -> AppResult<()> {
        if self.0? == club_id {
            Ok(())
        } else {
            Err(AppError::from(
                StatusCode::UNAUTHORIZED,
                "wrong credentials",
            ))
        }
    }
}
