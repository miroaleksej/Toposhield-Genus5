// tests/integration_test.rs
// End-to-end integration test for TopoShield ZKP system
// Verifies full lifecycle: keygen → sign → prove → verify

use toposhield::{manifold::HyperbolicManifold, witness::Witness, prover::TopoShieldProver};

#[test]
fn test_toposhield_full_lifecycle() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize manifold (sanity check)
    let manifold = HyperbolicManifold::new();
    assert_eq!(manifold.genus, 5);
    assert_eq!(manifold.chi, -8);
    assert_eq!(manifold.p_inv, 12345);

    // 2. Create prover (loads Circom artifacts and KZG setup)
    let prover = TopoShieldProver::new()?;

    // 3. Generate witness (signing)
    let message = b"Topological Cryptography Integration Test";
    let private_seed = b"integration_test_seed_2025";
    let witness = Witness::new(message, private_seed);

    // 4. Validate witness consistency
    assert_eq!(witness.gamma.len(), 20);
    assert_eq!(witness.delta.len(), 20);
    assert!(witness.gamma.iter().all(|&x| x < 20));
    assert!(witness.delta.iter().all(|&x| x < 20));

    // Verify determinants
    let det_pub = witness.h_pub[0] * witness.h_pub[3] - witness.h_pub[1] * witness.h_pub[2];
    let det_sig = witness.h_sig[0] * witness.h_sig[3] - witness.h_sig[1] * witness.h_sig[2];
    assert_eq!(det_pub, halo2_proofs::halo2curves::bn256::Fr::one());
    assert_eq!(det_sig, halo2_proofs::halo2curves::bn256::Fr::one());

    // 5. Generate ZK proof
    let proof = prover.prove(witness.clone())?;
    assert!(!proof.is_empty(), "Proof must be non-empty");
    assert!(proof.len() > 2000 && proof.len() < 3000, "Proof size should be ~2.3 KB");

    // 6. Verify proof
    let is_valid = prover.verify(
        &proof,
        witness.h_pub,
        witness.h_sig,
        witness.desc_m,
        witness.m_hash,
    )?;

    assert!(is_valid, "Proof must verify successfully");

    // 7. Tamper test: modify public key → proof must fail
    let mut tampered_h_pub = witness.h_pub;
    tampered_h_pub[0] += halo2_proofs::halo2curves::bn256::Fr::one();
    let is_invalid = prover.verify(
        &proof,
        tampered_h_pub,
        witness.h_sig,
        witness.desc_m,
        witness.m_hash,
    )?;

    assert!(!is_invalid, "Tampered proof must fail verification");

    Ok(())
}

#[test]
fn test_deterministic_witness_generation() {
    let message = b"Same message";
    let seed = b"same_seed";

    let w1 = Witness::new(message, seed);
    let w2 = Witness::new(message, seed);

    assert_eq!(w1.gamma, w2.gamma);
    assert_eq!(w1.delta, w2.delta);
    assert_eq!(w1.h_pub, w2.h_pub);
    assert_eq!(w1.h_sig, w2.h_sig);
    assert_eq!(w1.desc_m, w2.desc_m);
    assert_eq!(w1.m_hash, w2.m_hash);
}

#[test]
fn test_different_messages_produce_different_signatures() {
    let seed = b"fixed_seed";
    let w1 = Witness::new(b"Message 1", seed);
    let w2 = Witness::new(b"Message 2", seed);

    // Public keys should be the same (same gamma)
    assert_eq!(w1.h_pub, w2.h_pub);

    // Signatures must differ (different delta)
    assert_ne!(w1.h_sig, w2.h_sig);
    assert_ne!(w1.delta, w2.delta);
}
