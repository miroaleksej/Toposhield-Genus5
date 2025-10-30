// tests/integration_test.rs
// Full integration test for TopoShield ZKP (genus = 5)

use toposhield_zkp::{
    manifold::HyperbolicManifold,
    prover::TopoShieldProver,
    witness::Witness,
};
use halo2_proofs::halo2curves::bn256::Fr;

#[test]
fn test_toposhield_full_lifecycle() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize prover
    let prover = TopoShieldProver::new()?;
    
    // 2. Generate witness with known message and seed
    let message = b"Topological Cryptography Integration Test";
    let private_seed = b"integration_test_seed_2025";
    let witness = Witness::new(message, private_seed);
    
    // 3. Verify public key holonomy has det = 1
    let det_pub = witness.h_pub[0] * witness.h_pub[3] - witness.h_pub[1] * witness.h_pub[2];
    assert_eq!(det_pub, Fr::one(), "Public key holonomy must have det = 1");
    
    // 4. Verify signature holonomy has det = 1
    let det_sig = witness.h_sig[0] * witness.h_sig[3] - witness.h_sig[1] * witness.h_sig[2];
    assert_eq!(det_sig, Fr::one(), "Signature holonomy must have det = 1");
    
    // 5. Verify path validity
    assert_eq!(witness.gamma.len(), 20, "Gamma path must have length 20");
    assert_eq!(witness.delta.len(), 20, "Delta path must have length 20");
    assert!(witness.gamma.iter().all(|&x| x < 20), "Gamma indices must be in 0-19");
    assert!(witness.delta.iter().all(|&x| x < 20), "Delta indices must be in 0-19");
    
    // 6. Generate proof
    let proof = prover.prove(witness.clone())?;
    assert!(!proof.is_empty(), "Proof must be non-empty");
    assert!(proof.len() > 2000 && proof.len() < 3000, "Proof size must be ~2.3 KB");
    
    // 7. Verify proof
    let is_valid = prover.verify(
        &proof,
        witness.h_pub,
        witness.h_sig,
        witness.desc_m,
        witness.m_hash,
    )?;
    assert!(is_valid, "Proof verification must succeed");
    
    // 8. Test determinism: same inputs → same outputs
    let witness2 = Witness::new(message, private_seed);
    assert_eq!(witness.gamma, witness2.gamma, "Gamma must be deterministic");
    assert_eq!(witness.delta, witness2.delta, "Delta must be deterministic");
    assert_eq!(witness.h_pub, witness2.h_pub, "H_pub must be deterministic");
    assert_eq!(witness.h_sig, witness2.h_sig, "H_sig must be deterministic");
    
    // 9. Test manifold consistency
    let manifold = HyperbolicManifold::new();
    assert_eq!(witness.desc_m, Witness::compute_desc_m(manifold.p_inv), "Desc_M must match manifold");
    
    Ok(())
}

#[test]
fn test_invalid_proof_rejection() -> Result<(), Box<dyn std::error::Error>> {
    let prover = TopoShieldProver::new()?;
    
    // Valid witness
    let witness = Witness::new(b"Test", b"seed");
    let proof = prover.prove(witness.clone())?;
    
    // Tamper with public key
    let mut tampered_h_pub = witness.h_pub;
    tampered_h_pub[0] += Fr::one(); // Corrupt first element
    
    // Verification should fail
    let is_valid = prover.verify(
        &proof,
        tampered_h_pub,
        witness.h_sig,
        witness.desc_m,
        witness.m_hash,
    )?;
    assert!(!is_valid, "Verification must fail for tampered public key");
    
    Ok(())
}

#[test]
fn test_manifold_verification() -> Result<(), Box<dyn std::error::Error>> {
    let manifold = HyperbolicManifold::new();
    
    // Verify genus and chi
    assert_eq!(manifold.genus, 5, "Genus must be 5");
    assert_eq!(manifold.chi, -8, "Euler characteristic must be 2 - 2*5 = -8");
    
    // Verify all generators have det = 1
    for i in 0..10 {
        let (a, b, c, d) = manifold.get_generator(i);
        let det = a * d - b * c;
        assert_eq!(det, Fr::one(), "Generator {} must have det = 1", i);
    }
    
    // Verify inverses
    for i in 0..10 {
        let (a, b, c, d) = manifold.get_generator(i);
        let (a_inv, b_inv, c_inv, d_inv) = manifold.get_generator(i + 10);
        
        // M * M^{-1} = I
        let i11 = a * a_inv + b * c_inv;
        let i12 = a * b_inv + b * d_inv;
        let i21 = c * a_inv + d * c_inv;
        let i22 = c * b_inv + d * d_inv;
        
        assert_eq!(i11, Fr::one(), "M * M^(-1)[0,0] must be 1");
        assert_eq!(i12, Fr::zero(), "M * M^(-1)[0,1] must be 0");
        assert_eq!(i21, Fr::zero(), "M * M^(-1)[1,0] must be 0");
        assert_eq!(i22, Fr::one(), "M * M^(-1)[1,1] must be 1");
    }
    
    Ok(())
}
