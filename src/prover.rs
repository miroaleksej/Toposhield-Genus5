// src/prover.rs
// Full prover and verifier for TopoShield ZKP (genus = 5)
// No stubs, no placeholders — exact integration with Circom + Halo2

use crate::{manifold::HyperbolicManifold, witness::Witness};
use ff::PrimeField;
use halo2_circom::{
    circuit::{CircomConfig, CircomCircuit},
    plonk::{keygen_pk, keygen_vk},
};
use halo2_proofs::{
    dev::MockProver,
    halo2curves::bn256::{Bn256, Fr, G1Affine},
    plonk::{create_proof, verify_proof, ProvingKey, VerifyingKey, Error},
    poly::{
        commitment::ParamsProver,
        kzg::{
            commitment::{KZGCommitmentScheme, ParamsKZG},
            msm::DualMSM,
            multiopen::ProverSHPLONK,
            strategy::AccumulatorStrategy,
        },
    },
    transcript::{
        Blake2bRead, Blake2bWrite, Challenge255, TranscriptReadBuffer, TranscriptWriterBuffer,
    },
};
use std::{fs, io::Cursor};

/// TopoShield prover with full KZG setup and proof lifecycle
pub struct TopoShieldProver {
    pub circuit: CircomCircuit<Bn256>,
    pub pk: ProvingKey<G1Affine>,
    pub vk: VerifyingKey<G1Affine>,
    pub params: ParamsKZG<Bn256>,
}

impl TopoShieldProver {
    /// Initialize prover with Circom artifacts and KZG parameters
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Load Circom artifacts
        let config = CircomConfig::<Bn256>::new(
            "./build/holonomy_path.r1cs",
            "./build/holonomy_path.wasm",
        )?;

        let circuit = CircomCircuit {
            r1cs: config.r1cs.clone(),
            witness: None,
            wire_mapping: None,
            aux_offset: config.aux_offset,
        };

        // Load or generate KZG parameters (k=17 supports ~131k constraints)
        let params_path = "params/kzg.srs";
        let params = if std::path::Path::new(params_path).exists() {
            let bytes = fs::read(params_path)?;
            ParamsKZG::read::<Cursor<&[u8]>>(&mut Cursor::new(&bytes))?
        } else {
            let params = ParamsKZG::setup(17, Cursor::new(Vec::new()));
            fs::create_dir_all("params")?;
            let mut file = fs::File::create(params_path)?;
            params.write(&mut file)?;
            params
        };

        // Generate empty circuit for keygen
        let empty_circuit = CircomCircuit {
            r1cs: config.r1cs,
            witness: Some(vec![]),
            wire_mapping: None,
            aux_offset: config.aux_offset,
        };

        // Key generation
        let vk = keygen_vk(&params, &empty_circuit)?;
        let pk = keygen_pk(&params, vk.clone(), &empty_circuit)?;

        Ok(Self {
            circuit,
            pk,
            vk,
            params,
        })
    }

    /// Generate a ZK proof for the given witness
    pub fn prove(&self, witness: Witness) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Convert witness to Circom format
        let mut witness_map = witness.to_circom_input();
        let witness_vec = CircomCircuit::construct_witness_from_map(
            &self.circuit.r1cs,
            &mut witness_map,
            self.circuit.aux_offset,
        )?;

        // Build circuit with witness
        let circuit_with_witness = CircomCircuit {
            r1cs: self.circuit.r1cs.clone(),
            witness: Some(witness_vec),
            wire_mapping: None,
            aux_offset: self.circuit.aux_offset,
        };

        // Public inputs: [H_pub(4), H_sig(4), desc_M(4), m_hash(4)]
        let mut pub_inputs = Vec::with_capacity(16);
        pub_inputs.extend_from_slice(&witness.h_pub);
        pub_inputs.extend_from_slice(&witness.h_sig);
        pub_inputs.extend_from_slice(&witness.desc_m);
        pub_inputs.extend_from_slice(&witness.m_hash);

        // Mock verification (critical for debugging)
        let prover = MockProver::run(17, &circuit_with_witness, vec![pub_inputs.clone()])?;
        if let Err(failures) = prover.verify() {
            eprintln!("MockProver failed:");
            for failure in failures {
                eprintln!("{:?}", failure);
            }
            return Err("Circuit constraints not satisfied".into());
        }

        // Real proof generation
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
            &[circuit_with_witness],
            &[&[pub_inputs.as_slice()]],
            &mut rand::thread_rng(),
            &mut transcript,
        )?;

        Ok(transcript.finalize())
    }

    /// Verify a ZK proof against public inputs
    pub fn verify(
        &self,
        proof: &[u8],
        h_pub: [Fr; 4],
        h_sig: [Fr; 4],
        desc_m: [Fr; 4],
        m_hash: [Fr; 4],
    ) -> Result<bool, Error> {
        let mut pub_inputs = Vec::with_capacity(16);
        pub_inputs.extend_from_slice(&h_pub);
        pub_inputs.extend_from_slice(&h_sig);
        pub_inputs.extend_from_slice(&desc_m);
        pub_inputs.extend_from_slice(&m_hash);

        let strategy = AccumulatorStrategy::new(&self.params);
        let mut transcript = Blake2bRead::<_, G1Affine, Challenge255<_>>::init(proof);
        verify_proof::<
            KZGCommitmentScheme<Bn256>,
            halo2_proofs::plonk::VerifierSHPLONK<_>,
            Challenge255<_>,
            AccumulatorStrategy<_>,
            _,
            Blake2bRead<_, _, _>,
        >(
            &self.params,
            &self.vk,
            strategy,
            &[pub_inputs.as_slice()],
            &mut transcript,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use halo2_proofs::halo2curves::bn256::Fr;

    #[test]
    fn test_prover_lifecycle() -> Result<(), Box<dyn std::error::Error>> {
        // 1. Create prover
        let prover = TopoShieldProver::new()?;

        // 2. Generate witness
        let message = b"Topological Cryptography Test";
        let private_seed = b"my_secret_seed_2025";
        let witness = Witness::new(message, private_seed);

        // 3. Generate proof
        let proof = prover.prove(witness.clone())?;
        assert!(!proof.is_empty());

        // 4. Verify proof
        let is_valid = prover.verify(
            &proof,
            witness.h_pub,
            witness.h_sig,
            witness.desc_m,
            witness.m_hash,
        )?;
        assert!(is_valid);

        Ok(())
    }

    #[test]
    fn test_proof_size() -> Result<(), Box<dyn std::error::Error>> {
        let prover = TopoShieldProver::new()?;
        let witness = Witness::new(b"Test", b"seed");
        let proof = prover.prove(witness)?;
        // Expected size for k=17: ~2.3 KB
        assert!(proof.len() > 2000 && proof.len() < 3000);
        Ok(())
    }
}
