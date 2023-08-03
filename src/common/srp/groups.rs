//! Groups from [RFC 5054](https://tools.ietf.org/html/rfc5054)
//!
//! It is strongly recommended to use them instead of custom generated
//! groups. Additionally it is not recommended to use `G_1024` and `G_1536`,
//! they are provided only for compatibility with the legacy software.
use lazy_static::lazy_static;
use num_bigint::BigUint;
use ring::digest::{Algorithm, SHA256};

/// Group used for SRP computations
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SrpGroup {
    /// A large safe prime (N = 2q+1, where q is prime)
    pub n: BigUint,
    /// A generator modulo N
    pub g: BigUint,
    pub alg: &'static Algorithm,
}

lazy_static! {
    pub static ref G_2048: SrpGroup = SrpGroup {
        n: BigUint::from_bytes_be(include_bytes!("groups/2048.bin")),
        g: BigUint::from_bytes_be(&[2]),
        alg: &SHA256
    };
}

lazy_static! {
    pub static ref G_3072: SrpGroup = SrpGroup {
        n: BigUint::from_bytes_be(include_bytes!("groups/3072.bin")),
        g: BigUint::from_bytes_be(&[5]),
        alg: &SHA256
    };
}

lazy_static! {
    pub static ref G_4096: SrpGroup = SrpGroup {
        n: BigUint::from_bytes_be(include_bytes!("groups/4096.bin")),
        g: BigUint::from_bytes_be(&[5]),
        alg: &SHA256
    };
}

lazy_static! {
    pub static ref G_6144: SrpGroup = SrpGroup {
        n: BigUint::from_bytes_be(include_bytes!("groups/6144.bin")),
        g: BigUint::from_bytes_be(&[5]),
        alg: &SHA256
    };
}

lazy_static! {
    pub static ref G_8192: SrpGroup = SrpGroup {
        n: BigUint::from_bytes_be(include_bytes!("groups/8192.bin")),
        g: BigUint::from_bytes_be(&[19]),
        alg: &SHA256
    };
}
