use std::{convert::AsRef, str::FromStr};

use chrono::{
    serde::ts_microseconds::{deserialize as deserialize_to_ts, serialize as serialize_to_ts},
    DateTime, Duration, Utc,
};
use jsonwebtoken::{DecodingKey, EncodingKey};
use openssl::nid::Nid;
use ring::{
    rand::{SecureRandom, SystemRandom},
    signature::Ed25519KeyPair,
};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter};

use super::utils::gen_id;
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Claims {
    sub: Option<String>,
    iss: Option<String>,
    aud: Vec<String>,
    #[serde(
        serialize_with = "serialize_to_ts",
        deserialize_with = "deserialize_to_ts"
    )]
    nbf: DateTime<Utc>,
    #[serde(
        serialize_with = "serialize_to_ts",
        deserialize_with = "deserialize_to_ts"
    )]
    exp: DateTime<Utc>,
    #[serde(
        serialize_with = "serialize_to_ts",
        deserialize_with = "deserialize_to_ts"
    )]
    iat: DateTime<Utc>,
}

impl Claims {
    pub fn new(subject: String, expires_in: Duration) -> Self {
        Claims {
            sub: Some(subject),
            exp: Utc::now() + expires_in,
            ..Default::default()
        }
    }
}

pub struct JwKPair<'a> {
    alg: &'a Algorithm,
    inner: Vec<u8>,
}

impl<'a> JwKPair<'a> {
    pub fn new(alg: &'a Algorithm, inner: Vec<u8>) -> Self {
        JwKPair { alg, inner }
    }
}

impl<'a> JwKPair<'a> {
    pub fn export_alg(&self) -> Result<jsonwebtoken::Algorithm> {
        match self.alg {
            Algorithm::ED => Ok(jsonwebtoken::Algorithm::EdDSA),
            _ => {
                let mut alg = self.alg.as_ref().to_string();
                alg.push_str(self.inner.len().to_string().as_str());
                Ok(
                    jsonwebtoken::Algorithm::from_str(alg.as_str()).map_err(|_| {
                        JwtErorr::UnSupported(format!("export {} ed field failed", alg))
                    })?,
                )
            }
        }
    }
}

#[derive(EnumIter, AsRefStr, Debug, PartialEq)]
pub enum Algorithm {
    #[strum(serialize = "AES")]
    Aes,
    #[strum(serialize = "RSA")]
    Rsa,
    #[strum(serialize = "EC")]
    EC,
    #[strum(serialize = "PS")]
    PS,
    #[strum(serialize = "ED")]
    ED,
}

type Result<T> = std::result::Result<T, JwtErorr>;

#[derive(thiserror::Error, Debug)]
pub enum JwtErorr {
    #[error("kid is nonexistant")]
    NotFoundKid,
    #[error("unsupported {0}")]
    UnSupported(String),
    #[error("generate {alg} failed({stage})")]
    KeyGenErorr { alg: String, stage: String },
    #[error("verify jwt error {0}")]
    VerifyError(String),
    #[error("sign token erorr {0}")]
    SignError(String),
}

// 根据指定的算法生成一个 Key, ed 相关没有
pub fn genrate_key(alg: &Algorithm, size: usize) -> Result<JwKPair> {
    let secret = match alg {
        Algorithm::Aes => {
            let rng = SystemRandom::new();
            let mut dest = vec![0; size / 8];
            rng.fill(&mut dest).map_err(|_e| JwtErorr::KeyGenErorr {
                alg: alg.as_ref().to_string(),
                stage: String::from("filling"),
            })?;
            dest
        }
        Algorithm::Rsa | Algorithm::PS => openssl::rsa::Rsa::generate((size * 8) as u32)
            .map_err(|_| JwtErorr::KeyGenErorr {
                alg: alg.as_ref().to_string(),
                stage: String::from("generate"),
            })?
            .private_key_to_der()
            .map_err(|_| JwtErorr::KeyGenErorr {
                alg: alg.as_ref().to_string(),
                stage: String::from("fromat"),
            })?,
        Algorithm::EC => {
            let curve_name = match size {
                256 => Nid::X9_62_PRIME256V1,
                384 => Nid::SECP384R1,
                512 => Nid::SECP521R1,
                _ => {
                    return Err(JwtErorr::UnSupported(format!(
                        "unsupported ec algorithm block size({size})"
                    )))?
                }
            };
            let curve_group = openssl::ec::EcGroup::from_curve_name(curve_name).unwrap();
            openssl::ec::EcKey::generate(&curve_group)
                .map_err(|_| JwtErorr::KeyGenErorr {
                    alg: alg.as_ref().to_string(),
                    stage: String::from("generate"),
                })?
                .private_key_to_der()
                .map_err(|_| JwtErorr::KeyGenErorr {
                    alg: alg.as_ref().to_string(),
                    stage: String::from("format"),
                })?
        }
        Algorithm::ED => Ed25519KeyPair::generate_pkcs8(&SystemRandom::new())
            .map_err(|_| JwtErorr::KeyGenErorr {
                alg: alg.as_ref().to_string(),
                stage: String::from("generate"),
            })?
            .as_ref()
            .to_vec(),
    };
    Ok(JwKPair::new(alg, secret.clone()))
}

pub fn generate_jws(claims: &Claims, secret: &JwKPair) -> Result<String> {
    let mut headers = jsonwebtoken::Header::new(secret.export_alg()?);
    let encoding_key = match secret.alg {
        Algorithm::Aes => EncodingKey::from_secret(&secret.inner),
        Algorithm::Rsa | Algorithm::PS => EncodingKey::from_rsa_der(&secret.inner),
        Algorithm::EC => EncodingKey::from_ec_der(&secret.inner),
        Algorithm::ED => EncodingKey::from_ed_der(&secret.inner),
    };
    headers.kid = Some(gen_id(16));
    jsonwebtoken::encode(&headers, claims, &encoding_key)
        .map_err(|_| JwtErorr::SignError(format!("generate jws failed")))
}

pub fn verify_jws(
    token: &str,
    secret: &JwKPair,
    validation: jsonwebtoken::Validation,
) -> Result<Claims> {
    let decoding_key = match secret.alg {
        Algorithm::Aes => DecodingKey::from_secret(&secret.inner),
        Algorithm::Rsa | Algorithm::PS => DecodingKey::from_rsa_der(&secret.inner),
        Algorithm::EC => DecodingKey::from_ec_der(&secret.inner),
        Algorithm::ED => DecodingKey::from_ed_der(&secret.inner),
    };

    let jsonwebtoken::TokenData { header: _, claims } =
        jsonwebtoken::decode::<Claims>(token, &decoding_key, &validation)
            .map_err(|e| JwtErorr::VerifyError(format!("{}", e)))?;

    Ok(claims)
}
