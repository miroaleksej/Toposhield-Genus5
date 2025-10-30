// src/bin/prove-example.rs
// Example: generate a TopoShield ZK proof for a sample message
use std::fs;
use toposhield::{witness::Witness, prover::TopoShieldProver};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create prover (loads Circom artifacts and KZG setup)
    let prover = TopoShieldProver::new()?;

    // 2. Generate witness (signing)
    let message = b"TopoShield proof example — genus=5, enhanced ZKP";
    let private_seed = b"example_seed_2025";
    let witness = Witness::new(message, private_seed);

    // 3. Generate ZK proof
    let proof = prover.prove(witness)?;

    // 4. Save proof to disk
    fs::write("proof.bin", &proof)?;
    println!("✅ Proof saved to proof.bin ({} bytes)", proof.len());

    // 5. Optional: verify proof
    let is_valid = prover.verify(
        &proof,
        witness.h_pub,
        witness.h_sig,
        witness.desc_m,
        witness.m_hash,
    )?;
    println!("✅ Proof verification: {}", if is_valid { "SUCCESS" } else { "FAILED" });

    Ok(())
}
