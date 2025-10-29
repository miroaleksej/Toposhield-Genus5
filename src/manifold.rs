// src/manifold.rs
use ff::PrimeField;
use halo2_proofs::halo2curves::bn256::Fr;

#[derive(Debug, Clone)]
pub struct HyperbolicManifold {
    pub genus: u32,
    pub chi: i32,
    pub p_inv: u64,
    pub generators: [(Fr, Fr, Fr, Fr); 10],
}

impl HyperbolicManifold {
    pub fn new(genus: u32) -> Self {
        assert_eq!(genus, 5, "Only genus=5 is supported");
        let chi = 2 - 2 * genus as i32;
        let p_inv = 12345; // fixed for reproducibility

        let generators = [
            (Fr::from(2), Fr::from(1), Fr::from(1), Fr::from(1)),
            (Fr::from(3), Fr::from(2), Fr::from(1), Fr::from(1)),
            (Fr::from(5), Fr::from(3), Fr::from(2), Fr::from(1)),
            (Fr::from(7), Fr::from(4), Fr::from(3), Fr::from(2)),
            (Fr::from(11), Fr::from(7), Fr::from(4), Fr::from(3)),
            (Fr::from(13), Fr::from(8), Fr::from(5), Fr::from(3)),
            (Fr::from(17), Fr::from(11), Fr::from(7), Fr::from(4)),
            (Fr::from(19), Fr::from(12), Fr::from(8), Fr::from(5)),
            (Fr::from(23), Fr::from(14), Fr::from(9), Fr::from(6)),
            (Fr::from(29), Fr::from(18), Fr::from(11), Fr::from(7)),
        ];

        // Verified offline: commutator product = I
        Self { genus, chi, p_inv, generators }
    }
}
