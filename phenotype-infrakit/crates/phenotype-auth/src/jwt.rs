use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm, decode_header, errors::Error as JwtError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub exp: usize,
}

#[derive(Debug, Error)]
pub enum JwtErrorExt {
    #[error("jwt error: {0}")]
    Jwt(#[from] JwtError),
    #[error("invalid header")]
    InvalidHeader,
}

pub struct JwtValidator {
    decoding_key: DecodingKey,
    alg: Algorithm,
}

impl JwtValidator {
    pub fn new_hs256(secret: &[u8]) -> Self {
        Self { decoding_key: DecodingKey::from_secret(secret), alg: Algorithm::HS256 }
    }

    pub fn verify(&self, token: &str) -> Result<JwtClaims, JwtErrorExt> {
        let token_data = decode::<JwtClaims>(token, &self.decoding_key, &Validation::new(self.alg))?;
        Ok(token_data.claims)
    }
}
