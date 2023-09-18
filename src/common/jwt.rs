use std::{convert::AsRef, fmt::Debug, str::FromStr, time::Duration};

use chrono::{
    serde::ts_microseconds::{deserialize as deserialize_to_ts, serialize as serialize_to_ts},
    DateTime, Utc,
};
use jsonwebtoken::{DecodingKey, EncodingKey};
use openssl::error::ErrorStack;
use ring::{
    error::KeyRejected,
    rand::{SecureRandom, SystemRandom},
    signature::{
        EcdsaKeyPair, EcdsaSigningAlgorithm, Ed25519KeyPair, KeyPair, RsaKeyPair,
        ECDSA_P256_SHA256_ASN1_SIGNING, ECDSA_P384_SHA384_ASN1_SIGNING,
    },
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use strum::{AsRefStr, EnumIter};

use super::{constant::TOKEN_ISSUER, utils::gen_id};
use crate::{
    common::constant::{Gander, TokenType},
    dto::user::UserProfile,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct IdToken {
    #[serde(flatten)]
    token: TokenCalims,
    email: String,
    picture: String,
    name: String,
    gander: Gander,
}

impl IdToken {
    pub fn new(client_id: &str, user: &UserProfile, expires_in: Duration) -> Self {
        Self {
            token: TokenCalims::new(
                format!("https://auth.heliannuuthus.com/issuer/{}", client_id),
                user.openid,
                client_id.to_string(),
                expires_in,
            ),
            email: user.email.unwrap(),
            picture: user.avatar,
            name:  user.nickname,
            gander: user.gander,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AccessToken {
    #[serde(flatten)]
    token: TokenCalims,
    azp: String,
    scope: Vec<String>,
}

impl AccessToken {
    pub fn new(
        subject: &str,
        audience: &str,
        azp: &str,
        expires_in: Duration,
        scope: Vec<String>,
    ) -> Self {
        Self {
            token: TokenCalims::new(
                format!("https://auth.heliannuuthus.com/issuer/{}", azp),
                subject.to_string(),
                audience.to_string(),
                expires_in,
            ),
            azp: azp.to_string(),
            scope,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TokenCalims {
    iss: String,
    sub: String,
    // audience using single resource server
    // https://datatracker.ietf.org/doc/html/draft-ietf-oauth-security-topics#name-access-token-privilege-rest
    aud: String,
    #[serde(
        serialize_with = "serialize_to_ts",
        deserialize_with = "deserialize_to_ts"
    )]
    exp: DateTime<Utc>,
    #[serde(
        serialize_with = "serialize_to_ts",
        deserialize_with = "deserialize_to_ts"
    )]
    nbf: DateTime<Utc>,
    #[serde(
        serialize_with = "serialize_to_ts",
        deserialize_with = "deserialize_to_ts"
    )]
    iat: DateTime<Utc>,
    jti: String,
}

impl TokenCalims {
    pub fn new(iss: String, sub: String, aud: String, expires_in: std::time::Duration) -> Self {
        Self {
            iss,
            sub,
            aud,
            exp: Utc::now() + chrono::Duration::from_std(expires_in).unwrap(),
            nbf: Utc::now() - chrono::Duration::minutes(5),
            iat: Utc::now(),
            jti: gen_id(24),
        }
    }
}

pub struct JwKPair {
    alg: JwtAlgorithm,
    inner: Vec<u8>,
}

impl JwKPair {
    pub fn new(alg: JwtAlgorithm, inner: Vec<u8>) -> Self {
        JwKPair { alg, inner }
    }
    pub fn export_alg(&self) -> Result<jsonwebtoken::Algorithm> {
        jsonwebtoken::Algorithm::from_str(self.alg.as_ref()).map_err(|e| {
            JwtErorr::UnSupported(format!(
                "export alg {} field failed: {}",
                self.alg.as_ref(),
                e
            ))
        })
    }

    pub fn export_prikey(&self) -> Vec<u8> {
        self.inner.clone()
    }

    pub fn export_pubkey(&self) -> Result<Vec<u8>> {
        Ok(match &self.alg.variant() {
            JwtAlgorithmVariant::Hmac => self.inner.clone(),
            JwtAlgorithmVariant::Rsa => RsaKeyPair::from_der(&self.inner)?
                .public_key()
                .as_ref()
                .to_vec(),

            JwtAlgorithmVariant::Ec => EcdsaKeyPair::from_pkcs8(
                JwtAlgorithm::compute_ecdsa_algorithm(&self.alg)?,
                &self.inner,
            )?
            .public_key()
            .as_ref()
            .to_vec(),
            JwtAlgorithmVariant::Ed => ring::signature::Ed25519KeyPair::from_pkcs8(&self.inner)?
                .public_key()
                .as_ref()
                .to_vec(),
        })
    }
}

pub enum JwtAlgorithmVariant {
    Hmac,
    Rsa,
    Ec,
    Ed,
}

#[derive(EnumIter, AsRefStr, Debug, PartialEq)]
pub enum JwtAlgorithm {
    HS256,
    HS384,
    HS512,

    ES256,
    ES384,

    RS256,
    RS384,
    RS512,

    PS256,
    PS384,
    PS512,

    EdDSA,
}

impl JwtAlgorithm {
    pub fn variant(&self) -> JwtAlgorithmVariant {
        match self {
            JwtAlgorithm::HS256 | JwtAlgorithm::HS384 | JwtAlgorithm::HS512 => {
                JwtAlgorithmVariant::Hmac
            }
            JwtAlgorithm::ES256 | JwtAlgorithm::ES384 => JwtAlgorithmVariant::Ec,
            JwtAlgorithm::RS256
            | JwtAlgorithm::RS384
            | JwtAlgorithm::RS512
            | JwtAlgorithm::PS256
            | JwtAlgorithm::PS384
            | JwtAlgorithm::PS512 => JwtAlgorithmVariant::Rsa,
            JwtAlgorithm::EdDSA => JwtAlgorithmVariant::Ed,
        }
    }
    pub fn size(&self) -> Option<u32> {
        match self {
            JwtAlgorithm::HS256 | JwtAlgorithm::ES256 => Some(256u32),
            JwtAlgorithm::HS384 | JwtAlgorithm::ES384 => Some(384u32),
            JwtAlgorithm::HS512 => Some(512u32),
            JwtAlgorithm::RS256 | JwtAlgorithm::PS256 => Some(2048u32),
            JwtAlgorithm::RS384 | JwtAlgorithm::PS384 => Some(3072u32),
            JwtAlgorithm::RS512 | JwtAlgorithm::PS512 => Some(4096u32),
            JwtAlgorithm::EdDSA => None, // No specific size for EdDSA
        }
    }

    pub fn compute_ecdsa_algorithm(alg: &JwtAlgorithm) -> Result<&'static EcdsaSigningAlgorithm> {
        match alg {
            JwtAlgorithm::ES256 => Ok(&ECDSA_P256_SHA256_ASN1_SIGNING),
            JwtAlgorithm::ES384 => Ok(&ECDSA_P384_SHA384_ASN1_SIGNING),
            _ => Err(JwtErorr::UnSupported(format!(
                "unsupported alg({:?}) to ecdsa",
                alg
            ))),
        }
    }
}

type Result<T> = std::result::Result<T, JwtErorr>;

#[derive(thiserror::Error, Debug)]
pub enum JwtErorr {
    #[error("unsupported {0}")]
    UnSupported(String),
    #[error("generate {alg} failed({stage})")]
    KeyGenErorr { alg: String, stage: String },
    #[error("openssl key format error")]
    ErrorStack(#[from] ErrorStack),
    #[error("ed key format error")]
    KeyRejected(#[from] KeyRejected),
    #[error("verify jwt error {0}")]
    VerifyError(String),
    #[error("sign token erorr {0}")]
    SignError(String),
}

// 根据指定的算法生成一个 Key, ed 相关没有
pub fn genrate_key(alg: JwtAlgorithm) -> Result<JwKPair> {
    let size = alg.size().unwrap_or_default();
    let secret = match alg.variant() {
        JwtAlgorithmVariant::Hmac => {
            let rng = SystemRandom::new();
            let mut dest = vec![0; (size / 8) as usize];
            rng.fill(&mut dest).map_err(|_e| JwtErorr::KeyGenErorr {
                alg: alg.as_ref().to_string(),
                stage: String::from("filling"),
            })?;
            dest
        }
        JwtAlgorithmVariant::Rsa => openssl::rsa::Rsa::generate(size)
            .map_err(|_| JwtErorr::KeyGenErorr {
                alg: alg.as_ref().to_string(),
                stage: String::from("generate"),
            })?
            .private_key_to_der()
            .map_err(|_| JwtErorr::KeyGenErorr {
                alg: alg.as_ref().to_string(),
                stage: String::from("fromat"),
            })?,
        JwtAlgorithmVariant::Ec => EcdsaKeyPair::generate_pkcs8(
            JwtAlgorithm::compute_ecdsa_algorithm(&alg)?,
            &SystemRandom::new(),
        )
        .map_err(|_e| JwtErorr::KeyGenErorr {
            alg: alg.as_ref().to_string(),
            stage: String::from("generate"),
        })?
        .as_ref()
        .to_vec(),
        JwtAlgorithmVariant::Ed => Ed25519KeyPair::generate_pkcs8(&SystemRandom::new())
            .map_err(|_| JwtErorr::KeyGenErorr {
                alg: alg.as_ref().to_string(),
                stage: String::from("generate"),
            })?
            .as_ref()
            .to_vec(),
    };
    Ok(JwKPair::new(alg, secret.clone()))
}

pub fn generate_jws<T: Serialize>(claims: &T, secret: &JwKPair) -> Result<String> {
    let mut headers = jsonwebtoken::Header::new(secret.export_alg()?);
    let encoding_key = match secret.alg.variant() {
        JwtAlgorithmVariant::Hmac => EncodingKey::from_secret(&secret.export_prikey()),
        JwtAlgorithmVariant::Rsa => EncodingKey::from_rsa_der(&secret.export_prikey()),
        JwtAlgorithmVariant::Ec => EncodingKey::from_ec_der(&secret.export_prikey()),
        JwtAlgorithmVariant::Ed => EncodingKey::from_ed_der(&secret.export_prikey()),
    };
    headers.kid = Some(gen_id(16));
    jsonwebtoken::encode(&headers, claims, &encoding_key)
        .map_err(|e| JwtErorr::SignError(format!("generate jws failed: {}", e)))
}

pub fn verify_jws<T: DeserializeOwned>(
    token: &str,
    secret: &JwKPair,
    validation: jsonwebtoken::Validation,
) -> Result<T> {
    let decoding_key = match secret.alg.variant() {
        JwtAlgorithmVariant::Hmac => DecodingKey::from_secret(&secret.export_pubkey()?),
        JwtAlgorithmVariant::Rsa => DecodingKey::from_rsa_der(&secret.export_pubkey()?),
        JwtAlgorithmVariant::Ec => DecodingKey::from_ec_der(&secret.export_pubkey()?),
        JwtAlgorithmVariant::Ed => DecodingKey::from_ed_der(&secret.export_pubkey()?),
    };

    let jsonwebtoken::TokenData { header: _, claims } =
        jsonwebtoken::decode::<T>(token, &decoding_key, &validation)
            .map_err(|e| JwtErorr::VerifyError(format!("{}", e)))?;

    Ok(claims)
}

pub fn validation(key: &JwKPair, audience: Vec<String>) -> Result<jsonwebtoken::Validation> {
    let mut validation = jsonwebtoken::Validation::new(key.export_alg()?);
    validation.set_audience(&audience);
    Ok(validation)
}
