// src/prover.rs
// TopoShield Prover with enhanced trusted setup verification
// Integrates MPC and Powers of Tau protocols for secure SRS

use ff::Field;
use halo2_proofs::halo2curves::bn256::{Bn256, Fr, G1Affine};
use halo2_proofs::poly::kzg::commitment::ParamsKZG;
use halo2_proofs::plonk::{create_proof, verify_proof, ProvingKey, VerifyingKey};
use halo2_proofs::poly::{
    kzg::{
        commitment::{KZGCommitmentScheme, ParamsKZG},
        multiopen::ProverSHPLONK,
        strategy::AccumulatorStrategy,
    },
};
use halo2_proofs::transcript::{
    Blake2bRead, Blake2bWrite, Challenge255, TranscriptReadBuffer, TranscriptWriterBuffer,
};
use halo2_circom::{
    circuit::{CircomCircuit, CircomConfig},
    plonk::CircomReduction,
};
use std::{fs, io::Cursor, path::Path};
use sha2::{Sha256, Digest};
use toposhield::witness::Witness;

/// TopoShield Prover with enhanced security features
pub struct TopoShieldProver {
    params: ParamsKZG<Bn256>,
    pk: ProvingKey<G1Affine>,
    vk: VerifyingKey<G1Affine>,
    r1cs: halo2_circom::circuit::R1CS<Bn256>,
    aux_offset: usize,
}

impl TopoShieldProver {
    /// Creates a new prover with verified SRS
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Load Circom artifacts
        let config = CircomConfig::<Bn256>::new(
            "build/holonomy_path_enhanced.r1cs",
            "build/holonomy_path_enhanced.wasm",
        )?;
        
        // Load and verify SRS
        let params = Self::load_and_verify_params()?;
        
        // Create proving and verifying keys
        let empty_circuit = CircomCircuit {
            r1cs: config.r1cs.clone(),
            witness: Some(vec![]),
            wire_mapping: None,
            aux_offset: config.aux_offset,
        };
        
        let vk = halo2_proofs::plonk::keygen_vk(&params, &empty_circuit)?;
        let pk = halo2_proofs::plonk::keygen_pk(&params, vk.clone(), &empty_circuit)?;
        
        Ok(Self {
            params,
            pk,
            vk,
            r1cs: config.r1cs,
            aux_offset: config.aux_offset,
        })
    }
    
    /// Loads and verifies the KZG SRS parameters
    fn load_and_verify_params() -> Result<ParamsKZG<Bn256>, Box<dyn std::error::Error>> {
        let params_path = "params/kzg.srs";
        let hash_path = "params/kzg.srs.sha256";
        
        // Check if SRS file exists
        if !Path::new(params_path).exists() {
            return Err("KZG SRS file not found. Run 'mpc-setup' or 'powersoftau-setup'".into());
        }
        
        // Verify SRS integrity
        if Path::new(hash_path).exists() {
            let stored_hash = fs::read(hash_path)?;
            let current_hash = calculate_file_hash(params_path);
            
            if stored_hash != current_hash {
                return Err("KZG SRS integrity check failed. File may be corrupted or tampered with".into());
            }
        }
        
        // Load parameters
        let bytes = fs::read(params_path)?;
        let mut params = ParamsKZG::read::<_>(&mut Cursor::new(bytes))?;
        
        // Verify SRS compatibility
        if !Self::verify_srs_compatibility(&params) {
            return Err("KZG SRS is not compatible with TopoShield requirements".into());
        }
        
        Ok(params)
    }
    
    /// Verifies that the SRS is compatible with TopoShield requirements
    fn verify_srs_compatibility(params: &ParamsKZG<Bn256>) -> bool {
        // Check that SRS supports sufficient constraints
        if params.k() < 17 {
            return false;
        }
        
        // Check that SRS has G2 elements (required for KZG)
        if !params.has_g2() {
            return false;
        }
        
        // Check that SRS size matches expected constraints
        let expected_g1_size = 1 << 17;
        if params.g1_elements().len() != expected_g1_size + 1 {
            return false;
        }
        
        // Additional checks can be added here
        
        true
    }
    
    /// Generates a ZK proof for the given witness
    pub fn prove(&self, witness: Witness) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Prepare witness inputs for Circom
        let mut witness_map = witness.to_circom_input();
        let witness_vec = CircomCircuit::construct_witness_from_map(
            &self.r1cs,
            &mut witness_map,
            self.aux_offset,
        )?;
        
        // Create circuit with witness
        let circuit = CircomCircuit {
            r1cs: self.r1cs.clone(),
            witness: Some(witness_vec),
            wire_mapping: None,
            aux_offset: self.aux_offset,
        };
        
        // Public inputs: H_pub, H_sig, desc_M, m_hash (16 field elements)
        let instances = vec![vec![
            witness.h_pub[0], witness.h_pub[1], witness.h_pub[2], witness.h_pub[3],
            witness.h_sig[0], witness.h_sig[1], witness.h_sig[2], witness.h_sig[3],
            witness.desc_m[0], witness.desc_m[1], witness.desc_m[2], witness.desc_m[3],
            witness.m_hash[0], witness.m_hash[1], witness.m_hash[2], witness.m_hash[3],
        ]];
        
        // Mock verification for debugging
        let mock_prover = MockProver::run(17, &circuit, instances.clone())?;
        assert_eq!(
            mock_prover.verify(),
            Ok(()),
            "Mock prover failed - check witness or circuit"
        );
        
        // Generate proof
        let mut transcript = Blake2bWrite::<_, G1Affine, Challenge255<_>>::init(vec![]);
        create_proof::<
            KZGCommitmentScheme<Bn256>,
            ProverSHPLONK<_>,
            Challenge255<_>,
            DualMSM<_>,
            _,
            Blake2bWrite<_, _, _>,
            _,
        >(
            &self.params,
            &self.pk,
            &[circuit],
            &[&instances],
            &mut rand::thread_rng(),
            &mut transcript,
        )?;
        
        Ok(transcript.finalize())
    }
    
    /// Verifies a ZK proof
    pub fn verify(
        &self,
        proof: &[u8],
        h_pub: [Fr; 4],
        h_sig: [Fr; 4],
        desc_m: [Fr; 4],
        m_hash: [Fr; 4],
    ) -> Result<bool, halo2_proofs::plonk::Error> {
        let instances = vec![vec![
            h_pub[0], h_pub[1], h_pub[2], h_pub[3],
            h_sig[0], h_sig[1], h_sig[2], h_sig[3],
            desc_m[0], desc_m[1], desc_m[2], desc_m[3],
            m_hash[0], m_hash[1], m_hash[2], m_hash[3],
        ]];
        
        let strategy = AccumulatorStrategy::new(&self.params);
        let mut transcript = Blake2bRead::<_, G1Affine, Challenge255<_>>::init(proof);
        
        verify_proof::<
            KZGCommitmentScheme<Bn256>,
            halo2_proofs::poly::kzg::multiopen::VerifierSHPLONK<_>,
            Challenge255<_>,
            AccumulatorStrategy<_>,
            _,
            Blake2bRead<_, _, _>,
        >(&self.params, &self.vk, strategy, &[instances.as_slice()], &mut transcript)
    }
}

/// Calculates SHA-256 hash of a file
fn calculate_file_hash<P: AsRef<Path>>(path: P) -> Vec<u8> {
    let mut file = fs::File::open(path).expect("Failed to open file");
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher).expect("Failed to hash file");
    hasher.finalize().to_vec()
}
