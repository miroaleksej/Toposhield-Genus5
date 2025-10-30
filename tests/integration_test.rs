// tests/integration_test.rs
// End-to-end integration test for Enhanced TopoShield ZKP system
// Verifies full lifecycle with structural validation (reduced paths, enhanced desc_M)
use toposhield::{manifold::HyperbolicManifold, witness::Witness, prover::TopoShieldProver};

#[test]
fn test_toposhield_full_lifecycle() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize manifold (sanity check)
    let manifold = HyperbolicManifold::new();
    assert_eq!(manifold.genus, 5);
    assert_eq!(manifold.chi, -8);
    assert_eq!(manifold.p_inv, 12345);

    // 2. Create prover (loads enhanced Circom artifacts and KZG setup)
    let prover = TopoShieldProver::new()?;

    // 3. Generate witness (signing)
    let message = b"Topological Cryptography Integration Test — Enhanced ZKP";
    let private_seed = b"integration_test_seed_2025";
    let witness = Witness::new(message, private_seed);

    // 4. Validate witness consistency
    assert_eq!(witness.gamma.len(), 20);
    assert_eq!(witness.delta.len(), 20);
    assert!(witness.gamma.iter().all(|&x| x < 20));
    assert!(witness.delta.iter().all(|&x| x < 20));

    // 5. Verify paths are reduced (no adjacent cancellations)
    for i in 0..witness.gamma.len() - 1 {
        let a = witness.gamma[i] as i32;
        let b = witness.gamma[i + 1] as i32;
        assert!(!((a >= 0 && a <= 4 && b == a + 10) ||
                  (a >= 5 && a <= 9 && b == a + 10) ||
                  (a >= 10 && a <= 14 && b == a - 10) ||
                  (a >= 15 && a <= 19 && b == a - 10)),
                "gamma must be reduced");
    }
    for i in 0..witness.delta.len() - 1 {
        let a = witness.delta[i] as i32;
        let b = witness.delta[i + 1] as i32;
        assert!(!((a >= 0 && a <= 4 && b == a + 10) ||
                  (a >= 5 && a <= 9 && b == a + 10) ||
                  (a >= 10 && a <= 14 && b == a - 10) ||
                  (a >= 15 && a <= 19 && b == a - 10)),
                "delta must be reduced");
    }

    // 6. Verify determinants
    let det_pub = witness.h_pub[0] * witness.h_pub[3] - witness.h_pub[1] * witness.h_pub[2];
    let det_sig = witness.h_sig[0] * witness.h_sig[3] - witness.h_sig[1] * witness.h_sig[2];
    assert_eq!(det_pub, halo2_proofs::halo2curves::bn256::Fr::one());
    assert_eq!(det_sig, halo2_proofs::halo2curves::bn256::Fr::one());

    // 7. Generate ZK proof
    let proof = prover.prove(witness.clone())?;
    assert!(!proof.is_empty(), "Proof must be non-empty");
    assert!(proof.len() > 2000 && proof.len() < 3000, "Proof size should be ~2.3 KB");

    // 8. Verify proof
    let is_valid = prover.verify(
        &proof,
        witness.h_pub,
        witness.h_sig,
        witness.desc_m,
        witness.m_hash,
    )?;
    assert!(is_valid, "Proof must verify successfully");

    // 9. Tamper test: modify public key → proof must fail
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

    // 10. Tamper test: modify desc_M → proof must fail
    let mut tampered_desc_m = witness.desc_m;
    tampered_desc_m[0] += halo2_proofs::halo2curves::bn256::Fr::one();
    let is_invalid_desc = prover.verify(
        &proof,
        witness.h_pub,
        witness.h_sig,
        tampered_desc_m,
        witness.m_hash,
    )?;
    assert!(!is_invalid_desc, "Tampered desc_M must fail verification");

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
    // But both must be reduced
    for i in 0..w1.delta.len() - 1 {
        let a = w1.delta[i] as i32;
        let b = w1.delta[i + 1] as i32;
        assert!(!((a >= 0 && a <= 4 && b == a + 10) ||
                  (a >= 5 && a <= 9 && b == a + 10) ||
                  (a >= 10 && a <= 14 && b == a - 10) ||
                  (a >= 15 && a <= 19 && b == a - 10)));
    }
}

#[test]
fn test_enhanced_desc_m_consistency() {
    let w = Witness::new(b"Desc test", b"desc_seed");
    // Recompute expected desc_M manually
    use halo2_proofs::halo2curves::bn256::Fr;
    use poseidon::{PoseidonHasher, Spec};
    let mut hasher = PoseidonHasher::<Fr, _, 4, 1>::new(Spec::new());
    hasher.update(&[
        Fr::from(5u64),        // genus
        -Fr::from(8u64),       // χ
        Fr::from(12345u64),    // p_inv
        Fr::from(3u64),        // tr(a1) = 2+1
        Fr::from(4u64),        // tr(b1) = 3+1
        Fr::from(6u64),        // tr(a2) = 5+1
        Fr::from(9u64),        // tr(b2) = 7+2
        Fr::from(14u64),       // tr(a3) = 11+3
        Fr::from(16u64),       // tr(b3) = 13+3
        Fr::from(21u64),       // tr(a4) = 17+4
        Fr::from(24u64),       // tr(b4) = 19+5
        Fr::from(29u64),       // tr(a5) = 23+6
        Fr::from(26u64),       // tr(b5) = 21+5
    ]);
    let expected = hasher.squeeze();
    let expected_desc = [expected[0], expected[1], expected[2], expected[3]];
    assert_eq!(w.desc_m, expected_desc, "desc_M must include generator traces");
}
