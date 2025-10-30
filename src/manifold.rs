// src/witness.rs
// Enhanced witness generator for TopoShield ZKP (genus = 5, path length = 20)
// Includes reduced-path enforcement and enhanced manifold descriptor
use ff::{Field, PrimeField};
use halo2_proofs::halo2curves::bn256::Fr;
use poseidon::{PoseidonHasher, Spec};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use crate::manifold::HyperbolicManifold;

/// Witness for TopoShield ZKP circuit (enhanced version)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Witness {
    /// Public inputs (4x4 field elements)
    pub h_pub: [Fr; 4],
    pub h_sig: [Fr; 4],
    pub desc_m: [Fr; 4],
    pub m_hash: [Fr; 4],
    /// Private witness (generator indices 0–19, guaranteed reduced)
    pub gamma: Vec<u8>,
    pub delta: Vec<u8>,
}

const PATH_LENGTH: usize = 20; // Matches holonomy_path_enhanced.circom

impl Witness {
    /// Generate a complete, structurally valid witness
    pub fn new(message: &[u8], private_seed: &[u8]) -> Self {
        // 1. Create manifold (genus=5, hardcoded)
        let manifold = HyperbolicManifold::new();

        // 2. Derive gamma path from message and private seed
        let gamma_seed = Self::derive_seed(b"gamma", message, private_seed);
        let mut gamma = Self::generate_path(&gamma_seed, PATH_LENGTH);
        Self::ensure_reduced_path(&mut gamma);

        // 3. Compute public key holonomy: H_pub = Hol(gamma)
        let h_pub = Self::compute_holonomy(&gamma, &manifold);

        // 4. Derive delta path from message and public key (RFC 6979-style)
        let mut pk_bytes = Vec::new();
        for &elem in &h_pub {
            pk_bytes.extend_from_slice(elem.to_repr().as_ref());
        }
        let delta_seed = Self::derive_seed(b"delta", message, &pk_bytes);
        let mut delta = Self::generate_path(&delta_seed, PATH_LENGTH);
        Self::ensure_reduced_path(&mut delta);

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
            let index = (u64::from_le_bytes(hash[0].to_repr()[..8].try_into().unwrap_or([0u8; 8])) % 20) as u8;
            path.push(index);
        }
        path
    }

    /// Enforce reduced form: remove adjacent inverse pairs (a, a⁻¹) or (b, b⁻¹)
    fn ensure_reduced_path(path: &mut Vec<u8>) {
        let mut i = 0;
        while i < path.len().saturating_sub(1) {
            let a = path[i] as i32;
            let b = path[i + 1] as i32;

            let is_cancel =
                // a_i followed by a_i⁻¹
                (a >= 0 && a <= 4 && b == a + 10) ||
                // b_i followed by b_i⁻¹
                (a >= 5 && a <= 9 && b == a + 10) ||
                // a_i⁻¹ followed by a_i
                (a >= 10 && a <= 14 && b == a - 10) ||
                // b_i⁻¹ followed by b_i
                (a >= 15 && a <= 19 && b == a - 10);

            if is_cancel {
                path.remove(i);
                path.remove(i);
                if i > 0 { i -= 1; }
            } else {
                i += 1;
            }
        }

        // Pad to PATH_LENGTH if needed (deterministically)
        while path.len() < PATH_LENGTH {
            let last = *path.last().unwrap_or(&0);
            path.push((last + 1) % 20);
        }

        // Truncate if somehow longer (should not happen)
        path.truncate(PATH_LENGTH);
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

    /// Compute enhanced manifold descriptor: Poseidon(5, -8, 12345, tr(a1), ..., tr(b5))
    fn compute_desc_m(p_inv: u64) -> [Fr; 4] {
        // Traces of the 10 positive generators (a1 to b5)
        let traces = [
            Fr::from(2 + 1),   // tr(a1) = 3
            Fr::from(3 + 1),   // tr(b1) = 4
            Fr::from(5 + 1),   // tr(a2) = 6
            Fr::from(7 + 2),   // tr(b2) = 9
            Fr::from(11 + 3),  // tr(a3) = 14
            Fr::from(13 + 3),  // tr(b3) = 16
            Fr::from(17 + 4),  // tr(a4) = 21
            Fr::from(19 + 5),  // tr(b4) = 24
            Fr::from(23 + 6),  // tr(a5) = 29
            Fr::from(21 + 5),  // tr(b5) = 26
        ];

        let mut hasher = PoseidonHasher::<Fr, _, 4, 1>::new(Spec::new());
        hasher.update(&[
            Fr::from(5u64),        // genus
            -Fr::from(8u64),       // χ = 2 - 2*5 = -8
            Fr::from(p_inv),
        ]);
        hasher.update(&traces);
        let result = hasher.squeeze();
        [result[0], result[1], result[2], result[3]]
    }

    /// Convert witness to Circom-compatible input format (hex strings for field elements)
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
    fn test_reduced_paths() {
        let w = Witness::new(b"Reduced Test", b"reduced_seed");
        // Check no adjacent cancellations
        for i in 0..w.gamma.len() - 1 {
            let a = w.gamma[i] as i32;
            let b = w.gamma[i + 1] as i32;
            assert!(!((a >= 0 && a <= 4 && b == a + 10) ||
                      (a >= 5 && a <= 9 && b == a + 10) ||
                      (a >= 10 && a <= 14 && b == a - 10) ||
                      (a >= 15 && a <= 19 && b == a - 10)));
        }
        for i in 0..w.delta.len() - 1 {
            let a = w.delta[i] as i32;
            let b = w.delta[i + 1] as i32;
            assert!(!((a >= 0 && a <= 4 && b == a + 10) ||
                      (a >= 5 && a <= 9 && b == a + 10) ||
                      (a >= 10 && a <= 14 && b == a - 10) ||
                      (a >= 15 && a <= 19 && b == a - 10)));
        }
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
