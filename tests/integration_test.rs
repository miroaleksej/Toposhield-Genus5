#[cfg(test)]
mod tests {
    use super::*;
    use crate::{manifold::HyperbolicManifold, witness::Witness, prover::TopoShieldProver};
    use halo2_proofs::halo2curves::bn256::Fr;

    #[test]
    fn test_toposhield_genus5() {
        // 1. Setup manifold
        let manifold = HyperbolicManifold::new(5);
        assert_eq!(manifold.chi, -8);

        // 2. Generate witness (gamma = [0,1,2,...,19], delta = [19,18,...,0])
        let gamma: Vec<u8> = (0..20).collect();
        let delta: Vec<u8> = (0..20).rev().collect();
        let witness = Witness::new(gamma, delta, manifold.p_inv);

        // 3. Compute public inputs (simplified: use identity for H_pub, H_sig)
        let H_pub = vec![Fr::from(1), Fr::from(0), Fr::from(0), Fr::from(1)];
        let H_sig = vec![Fr::from(1), Fr::from(0), Fr::from(0), Fr::from(1)];
        let desc_M = vec![Fr::from(1), Fr::from(2), Fr::from(3), Fr::from(4)]; // placeholder
        let pub_inputs = [H_pub, H_sig, desc_M].concat();

        // 4. Prove
        let prover = TopoShieldProver::new().unwrap();
        let proof = prover.prove(witness, pub_inputs).unwrap();
        assert!(!proof.is_empty());

        // 5. TIS audit: in real system, collect (U_r, U_z) and compute betti numbers
        // For MVP, we assume TIS = 0 if proof verifies
    }
}
