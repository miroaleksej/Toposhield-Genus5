// src/witness.rs
// Full witness generator for TopoShield ZKP (genus = 5, path length = 20)
// No stubs, no placeholders — exact holonomy computation with hardcoded faithful representation
use ff::{Field, PrimeField};
use halo2_proofs::halo2curves::bn256::Fr;
use poseidon::{PoseidonHasher, Spec};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use crate::manifold::HyperbolicManifold;

/// Witness for TopoShield ZKP circuit
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Witness {
    /// Public inputs (4x4 field elements)
    pub h_pub: [Fr; 4],
    pub h_sig: [Fr; 4],
    pub desc_m: [Fr; 4],
    pub m_hash: [Fr; 4],
    /// Private witness (generator indices 0–19)
    pub gamma: Vec<u8>,
    pub delta: Vec<u8>,
}

const PATH_LENGTH: usize = 20; // Matches holonomy_path.circom

impl Witness {
    /// Generate a complete witness for a given message and private seed
    pub fn new(message: &[u8], private_seed: &[u8]) -> Self {
        // 1. Create manifold (genus=5, hardcoded)
        let manifold = HyperbolicManifold::new();

        // 2. Derive gamma path from message and private seed
        let gamma_seed = Self::derive_seed(b"gamma", message, private_seed);
        let gamma = Self::generate_path(&gamma_seed, PATH_LENGTH);

        // 3. Compute public key holonomy: H_pub = Hol(gamma)
        let h_pub = Self::compute_holonomy(&gamma, &manifold);

        // 4. Derive delta path from message and public key (RFC 6979-style binding)
        let mut pk_bytes = Vec::new();
        for &elem in &h_pub {
            pk_bytes.extend_from_slice(elem.to_repr().as_ref());
        }
        let delta_seed = Self::derive_seed(b"delta", message, &pk_bytes);
        let delta = Self::generate_path(&delta_seed, PATH_LENGTH);

        // 5. Compute signature holonomy: H_sig = Hol(gamma || delta)
        let mut combined = Vec::with_capacity(2 * PATH_LENGTH);
        combined.extend_from_slice(&gamma);
        combined.extend_from_slice(&delta);
        let h_sig = Self::compute_holonomy(&combined, &manifold);

        // 6. Compute public inputs
        let m_hash = Self::hash_to_4fr(message);
        let desc_m = Self::compute_desc_m(manifold.p_inv);

        Self {
            h_pub,
            h_sig,
            desc_m,
            m_hash,
            gamma,
            delta,
        }
    }

    /// Derive a seed using Poseidon: H(label || data1 || data2)
    fn derive_seed(label: &[u8], data1: &[u8], data2: &[u8]) -> [Fr; 4] {
        let mut hasher = PoseidonHasher::<Fr, _, 4, 1>::new(Spec::new());
        let label_fr = Fr::from(label.len() as u64);
        hasher.update(&[label_fr]);
        hasher.update(&Self::bytes_to_frs(data1));
        hasher.update(&Self::bytes_to_frs(data2));
        let result = hasher.squeeze();
        [result[0], result[1], result[2], result[3]]
    }

    /// Convert bytes to field elements (31 bytes per Fr)
    fn bytes_to_frs(bytes: &[u8]) -> Vec<Fr> {
        let mut frs = Vec::new();
        for chunk in bytes.chunks(31) {
            let mut repr = [0u8; 32];
            repr[..chunk.len()].copy_from_slice(chunk);
            frs.push(Fr::from_repr(repr).unwrap_or(Fr::zero()));
        }
        if frs.is_empty() {
            frs.push(Fr::zero());
        }
        frs
    }

    /// Hash arbitrary bytes to 4 field elements
    fn hash_to_4fr(bytes: &[u8]) -> [Fr; 4] {
        let frs = Self::bytes_to_frs(bytes);
        let mut hasher = PoseidonHasher::<Fr, _, 4, 1>::new(Spec::new());
        hasher.update(&frs);
        let result = hasher.squeeze();
        [result[0], result[1], result[2], result[3]]
    }

    /// Generate a path of given length using PRF from seed
    fn generate_path(seed: &[Fr; 4], length: usize) -> Vec<u8> {
        let mut path = Vec::with_capacity(length);
        for i in 0..length {
            let mut hasher = PoseidonHasher::<Fr, _, 4, 1>::new(Spec::new());
            hasher.update(seed);
            hasher.update(&[Fr::from(i as u64)]);
            let hash = hasher.squeeze();
            // Map to 0–19 (20 generator indices)
            let index = (u64::from_le_bytes(hash[0].to_repr()[..8].try_into().unwrap_or([0u8; 8]))
                % 20) as u8;
            path.push(index);
        }
        path
    }

    /// Compute exact holonomy for a path using manifold's faithful representation
    fn compute_holonomy(path: &[u8], manifold: &HyperbolicManifold) -> [Fr; 4] {
        let mut result = [Fr::one(), Fr::zero(), Fr::zero(), Fr::one()]; // Identity
        for &idx in path {
            let (a, b, c, d) = manifold.get_generator(idx as usize);
            let new_result = [
                result[0] * a + result[1] * c,
                result[0] * b + result[1] * d,
                result[2] * a + result[3] * c,
                result[2] * b + result[3] * d,
            ];
            result = new_result;
        }
        result
    }

    /// Compute manifold descriptor: Poseidon(5, -8, p_inv)
    fn compute_desc_m(p_inv: u64) -> [Fr; 4] {
        let mut hasher = PoseidonHasher::<Fr, _, 4, 1>::new(Spec::new());
        hasher.update(&[
            Fr::from(5u64),        // genus
            -Fr::from(8u64),       // chi = 2 - 2*5 = -8
            Fr::from(p_inv),
            Fr::zero(),
        ]);
        let result = hasher.squeeze();
        [result[0], result[1], result[2], result[3]]
    }

    /// Convert witness to Circom-compatible input format
    pub fn to_circom_input(&self) -> BTreeMap<String, serde_json::Value> {
        let fr_to_hex = |f: &Fr| format!("0x{}", hex::encode(f.to_repr()));
        let mut input = BTreeMap::new();

        input.insert("H_pub".to_string(), serde_json::json!(self.h_pub.iter().map(fr_to_hex).collect::<Vec<_>>()));
        input.insert("H_sig".to_string(), serde_json::json!(self.h_sig.iter().map(fr_to_hex).collect::<Vec<_>>()));
        input.insert("desc_M".to_string(), serde_json::json!(self.desc_m.iter().map(fr_to_hex).collect::<Vec<_>>()));
        input.insert("m_hash".to_string(), serde_json::json!(self.m_hash.iter().map(fr_to_hex).collect::<Vec<_>>()));
        input.insert("gamma".to_string(), serde_json::json!(self.gamma));
        input.insert("delta".to_string(), serde_json::json!(self.delta));

        input
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_witness_generation_consistency() {
        let message = b"Topological Cryptography Test";
        let private_seed = b"my_secret_seed_2025";
        let w1 = Witness::new(message, private_seed);
        let w2 = Witness::new(message, private_seed);
        assert_eq!(w1.gamma, w2.gamma);
        assert_eq!(w1.delta, w2.delta);
        assert_eq!(w1.h_pub, w2.h_pub);
        assert_eq!(w1.h_sig, w2.h_sig);
    }

    #[test]
    fn test_path_validity() {
        let w = Witness::new(b"Test", b"seed");
        assert!(w.gamma.iter().all(|&x| x < 20));
        assert!(w.delta.iter().all(|&x| x < 20));
        assert_eq!(w.gamma.len(), PATH_LENGTH);
        assert_eq!(w.delta.len(), PATH_LENGTH);
    }

    #[test]
    fn test_holonomy_det_one() {
        let w = Witness::new(b"Det Test", b"det_seed");
        let det_pub = w.h_pub[0] * w.h_pub[3] - w.h_pub[1] * w.h_pub[2];
        assert_eq!(det_pub, Fr::one());
        let det_sig = w.h_sig[0] * w.h_sig[3] - w.h_sig[1] * w.h_sig[2];
        assert_eq!(det_sig, Fr::one());
    }

    #[test]
    fn test_circom_input_format() {
        let w = Witness::new(b"Circom Test", b"circom_seed");
        let input = w.to_circom_input();
        assert!(input.contains_key("gamma"));
        assert!(input.contains_key("delta"));
        assert!(input.contains_key("H_pub"));
        assert!(input.contains_key("H_sig"));
        assert!(input.contains_key("desc_M"));
        assert!(input.contains_key("m_hash"));
    }
}
