use num_bigint::BigUint;
use ring::digest::Context;

use crate::common::srp::groups::SrpGroup;

// u = H(PAD(A) | PAD(B))
pub fn compute_u(params: &SrpGroup, a_pub: &[u8], b_pub: &[u8]) -> BigUint {
    let mut u = Context::new(params.alg);
    u.update(a_pub);
    u.update(b_pub);
    BigUint::from_bytes_be(u.finish().as_ref())
}

// k = H(N | PAD(g))
pub fn compute_k(params: &SrpGroup) -> BigUint {
    let n = params.n.to_bytes_be();
    let g_bytes = params.g.to_bytes_be();
    let mut buf = vec![0u8; n.len()];
    let l = n.len() - g_bytes.len();
    buf[l..].copy_from_slice(&g_bytes);

    let mut d = Context::new(params.alg);
    d.update(&n);
    d.update(&buf);
    BigUint::from_bytes_be(d.finish().as_ref())
}

// M1 = H(A, B, K) this doesn't follow the spec but apparently no one does for M1
// M1 should equal =  H(H(N) XOR H(g) | H(U) | s | A | B | K) according to the spec
pub fn compute_m1(params: &SrpGroup, a_pub: &[u8], b_pub: &[u8], key: &[u8]) -> Vec<u8> {
    let mut d = Context::new(params.alg);
    d.update(a_pub);
    d.update(b_pub);
    d.update(key);
    d.finish().as_ref().to_vec()
}

// M2 = H(A, M1, K)
pub fn compute_m2(params: &SrpGroup, a_pub: &[u8], m1: &[u8], key: &[u8]) -> Vec<u8> {
    let mut d = Context::new(params.alg);
    d.update(a_pub);
    d.update(m1);
    d.update(key);
    d.finish().as_ref().to_vec()
}
