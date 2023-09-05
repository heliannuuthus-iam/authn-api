use num_bigint::BigUint;

use crate::common::srp::{
    errors::SrpError,
    groups::SrpGroup,
    utils::{compute_k, compute_m1, compute_m2, compute_u},
};

/// SRP server state
pub struct SrpServer<'a> {
    params: &'a SrpGroup,
}

/// SRP server state after handshake with the client.
#[derive(serde::Deserialize, serde::Serialize)]
pub struct SrpServerVerifier {
    m1: Vec<u8>,
    m2: Vec<u8>,
    key: Vec<u8>,
}

impl<'a> SrpServer<'a> {
    /// Create new server state.
    pub fn new(params: &'a SrpGroup) -> Self {
        Self { params }
    }

    //  k*v + g^b % N
    pub fn compute_b_pub(&self, b: &BigUint, k: &BigUint, v: &BigUint) -> BigUint {
        let inter = (k * v) % &self.params.n;
        (inter + self.params.g.modpow(b, &self.params.n)) % &self.params.n
    }

    // <premaster secret> = (A * v^u) ^ b % N
    pub fn compute_pre_master_secret(
        &self,
        a_pub: &BigUint,
        v: &BigUint,
        u: &BigUint,
        b: &BigUint,
    ) -> BigUint {
        // (A * v^u)
        let base = (a_pub * v.modpow(u, &self.params.n)) % &self.params.n;
        base.modpow(b, &self.params.n)
    }

    /// Get public ephemeral value for sending to the client.
    pub fn compute_public_ephemeral(&self, b: &[u8], v: &[u8]) -> Vec<u8> {
        self.compute_b_pub(
            &BigUint::from_bytes_be(b),
            &compute_k(self.params),
            &BigUint::from_bytes_be(v),
        )
        .to_bytes_be()
    }

    /// Process client reply to the handshake.
    /// b is a random value,
    /// v is the provided during initial user registration
    pub fn process_reply(
        &self,
        b: &[u8],
        v: &[u8],
        a_pub: &[u8],
    ) -> Result<SrpServerVerifier, SrpError> {
        let b = BigUint::from_bytes_be(b);
        let v = BigUint::from_bytes_be(v);
        let a_pub = BigUint::from_bytes_be(a_pub);

        let k = compute_k(self.params);
        let b_pub = self.compute_b_pub(&b, &k, &v);

        // Safeguard against malicious A
        if &a_pub % &self.params.n == BigUint::default() {
            return Err(SrpError::IllegalParameter("a_pub".to_owned()));
        }

        let u = compute_u(self.params, &a_pub.to_bytes_be(), &b_pub.to_bytes_be());

        let key = self.compute_pre_master_secret(&a_pub, &v, &u, &b);

        let m1 = compute_m1(
            self.params,
            &a_pub.to_bytes_be(),
            &b_pub.to_bytes_be(),
            &key.to_bytes_be(),
        );

        let m2 = compute_m2(self.params, &a_pub.to_bytes_be(), &m1, &key.to_bytes_be());

        Ok(SrpServerVerifier {
            m1,
            m2,
            key: key.to_bytes_be(),
        })
    }
}

impl SrpServerVerifier {
    /// Get shared secret between user and the server. (do not forget to verify
    /// that keys are the same!)
    pub fn key(&self) -> &[u8] {
        &self.key
    }

    /// Verification data for sending to the client.
    pub fn proof(&self) -> &[u8] {
        // TODO not Output
        self.m2.as_slice()
    }

    /// Process user proof of having the same shared secret.
    pub fn verify_client(&self, reply: &[u8]) -> Result<(), SrpError> {
        if self.m1 == reply {
            // aka == 0
            Ok(())
        } else {
            Err(SrpError::BadRecordMac("client".to_owned()))
        }
    }
}
