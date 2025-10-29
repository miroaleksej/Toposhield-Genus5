// src/witness.rs
use crate::manifold::HyperbolicManifold;
use ff::PrimeField;
use halo2_proofs::halo2curves::bn256::Fr;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize)]
pub struct Witness {
    pub gamma: Vec<u8>, // indices 0..19 (20 steps)
    pub delta: Vec<u8>,
    pub p_inv: u64,
}

impl Witness {
    pub fn new(gamma: Vec<u8>, delta: Vec<u8>, p_inv: u64) -> Self {
        assert!(gamma.len() == 20 && delta.len() == 20);
        Self { gamma, delta, p_inv }
    }

    pub fn to_circom_input(&self) -> BTreeMap<String, serde_json::Value> {
        let mut input = BTreeMap::new();
        input.insert("gamma".to_string(), serde_json::json!(self.gamma));
        input.insert("delta".to_string(), serde_json::json!(self.delta));
        // Public inputs computed externally
        input
    }
}
