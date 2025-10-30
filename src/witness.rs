// src/witness.rs
// Full witness generator for TopoShield ZKP (genus = 5)
// No stubs, no placeholders — exact holonomy computation

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

    /// Private witness (generator indices 0-19)
    pub gamma: Vec<u8>,
    pub delta: Vec<u8>,
}

impl Witness {
    /// Generate a complete witness for a given message and private seed
    pub fn new(message: &[u8], private_seed: &[u8]) -> Self {
        // 1. Create manifold (genus=5)
        let manifold = HyperbolicManifold::new();

        // 2. Hash message and seed to derive gamma path
        let gamma_seed = Self::derive_seed(b"gamma", message, private_seed);
        let gamma = Self::generate_path(&gamma_seed, 20);

        // 3. Compute public key holonomy
        let h_pub = Self::compute_holonomy(&gamma, &manifold);

        // 4. Derive delta path from message and public key
        let mut pk_bytes = Vec::new();
        for &elem in &h_pub {
            pk_bytes.extend_from_slice(elem.to_repr().as_ref());
        }
        let delta_seed = Self::derive_seed(b"delta", message, &pk_bytes);
        let delta = Self::generate_path(&delta_seed, 20);

        // 5. Compute signature holonomy (gamma || delta)
        let mut combined = Vec::with_capacity(40);
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

        // Convert data1 to field elements
        let data1_fr = Self::bytes_to_frs(data1);
        hasher.update(&data1_fr);

        // Convert data2 to field elements
        let data2_fr = Self::bytes_to_frs(data2);
        hasher.update(&data2_fr);

        let result = hasher.squeeze();
        let mut output = [Fr::zero(); 4];
        for (i, val) in result.iter().take(4).enumerate() {
            output[i] = *val;
        }
        output
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
        let mut output = [Fr::zero(); 4];
        for (i, val) in result.iter().take(4).enumerate() {
            output[i] = *val;
        }
        output
    }

    /// Generate a path of given length using PRF from seed
    fn generate_path(seed: &[Fr; 4], length: usize) -> Vec<u8> {
        let mut path = Vec::with_capacity(length);
        for i in 0..length {
            let mut hasher = PoseidonHasher::<Fr, _, 4, 1>::new(Spec::new());
            hasher.update(seed);
            hasher.update(&[Fr::from(i as u64)]);
            let hash = hasher.squeeze();
            // Map to 0-19 (20 generator indices)
            let index = (hash[0].to_repr()[0] % 20) as u8;
            path.push(index);
        }
        path
    }

    /// Compute exact holonomy for a path using manifold's faithful representation
    fn compute_holonomy(path: &[u8], manifold: &HyperbolicManifold) -> [Fr; 4] {
        // Start with identity matrix
        let mut result = [Fr::one(), Fr::zero(), Fr::zero(), Fr::one()];

        for &idx in path {
            let (a, b, c, d) = manifold.get_generator(idx as usize);
            // Matrix multiplication: result = result * generator
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
            Fr::zero(),            // padding to 4 elements
        ]);
        let result = hasher.squeeze();
        let mut output = [Fr::zero(); 4];
        for (i, val) in result.iter().take(4).enumerate() {
            output[i] = *val;
        }
        output
    }

    /// Convert witness to Circom-compatible input format
    pub fn to_circom_input(&self) -> BTreeMap<String, serde_json::Value> {
        let mut input = BTreeMap::new();

        // Public inputs (4 elements each)
        input.insert("H_pub".to_string(), serde_json::json!(self.h_pub.iter().map(|x| x.to_repr()).collect::<Vec<_>>()));
        input.insert("H_sig".to_string(), serde_json::json!(self.h_sig.iter().map(|x| x.to_repr()).collect::<Vec<_>>()));
        input.insert("desc_M".to_string(), serde_json::json!(self.desc_m.iter().map(|x| x.to_repr()).collect::<Vec<_>>()));
        input.insert("m_hash".to_string(), serde_json::json!(self.m_hash.iter().map(|x| x.to_repr()).collect::<Vec<_>>()));

        // Private witness (u8 vectors)
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
        let witness1 = Witness::new(message, private_seed);
        let witness2 = Witness::new(message, private_seed);

        // Deterministic generation: same inputs → same outputs
        assert_eq!(witness1.gamma, witness2.gamma);
        assert_eq!(witness1.delta, witness2.delta);
        assert_eq!(witness1.h_pub, witness2.h_pub);
        assert_eq!(witness1.h_sig, witness2.h_sig);
    }

    #[test]
    fn test_path_validity() {
        let message = b"Test";
        let private_seed = b"seed";
        let witness = Witness::new(message, private_seed);

        // All indices must be in 0-19
        assert!(witness.gamma.iter().all(|&x| x < 20));
        assert!(witness.delta.iter().all(|&x| x < 20));
        assert_eq!(witness.gamma.len(), 20);
        assert_eq!(witness.delta.len(), 20);
    }

    #[test]
    fn test_holonomy_det_one() {
        let message = b"Det Test";
        let private_seed = b"det_seed";
        let witness = Witness::new(message, private_seed);

        // Verify det(H_pub) = 1
        let det_pub = witness.h_pub[0] * witness.h_pub[3] - witness.h_pub[1] * witness.h_pub[2];
        assert_eq!(det_pub, Fr::one());

        // Verify det(H_sig) = 1
        let det_sig = witness.h_sig[0] * witness.h_sig[3] - witness.h_sig[1] * witness.h_sig[2];
        assert_eq!(det_sig, Fr::one());
    }

    #[test]
    fn test_circom_input_format() {
        let message = b"Circom Test";
        let private_seed = b"circom_seed";
        let witness = Witness::new(message, private_seed);
        let circom_input = witness.to_circom_input();

        // Must contain all required keys
        assert!(circom_input.contains_key("gamma"));
        assert!(circom_input.contains_key("delta"));
        assert!(circom_input.contains_key("H_pub"));
        assert!(circom_input.contains_key("H_sig"));
        assert!(circom_input.contains_key("desc_M"));
        assert!(circom_input.contains_key("m_hash"));
    }
}
