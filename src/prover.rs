// src/prover.rs
use crate::{manifold::HyperbolicManifold, witness::Witness};
use halo2_circom::circuit::{CircomConfig, CircomCircuit};
use halo2_proofs::{
    dev::MockProver,
    halo2curves::bn256::{Bn256, Fr, G1Affine},
    plonk::{create_proof, verify_proof, Error, ProvingKey, VerifyingKey},
    poly::{
        commitment::ParamsProver,
        kzg::{
            commitment::{KZGCommitmentScheme, ParamsKZG},
            msm::DualMSM,
            multiopen::ProverSHPLONK,
        },
    },
    transcript::{Blake2bWrite, Challenge255, TranscriptWriterBuffer},
};
use std::fs;

pub struct TopoShieldProver {
    circuit: CircomCircuit<Bn256>,
    pk: ProvingKey<G1Affine>,
    vk: VerifyingKey<G1Affine>,
    params: ParamsKZG<Bn256>,
}

impl TopoShieldProver {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = CircomConfig::<Bn256>::new(
            "./build/holonomy_chain.r1cs",
            "./build/holonomy_chain.wasm",
        )?;

        let circuit = CircomCircuit {
            r1cs: config.r1cs.clone(),
            witness: None,
            wire_mapping: None,
            aux_offset: config.aux_offset,
        };

        let params_path = "params/kzg.srs";
        let params = if std::path::Path::new(params_path).exists() {
            let bytes = fs::read(params_path)?;
            ParamsKZG::read(&mut &bytes[..])?
        } else {
            ParamsKZG::setup(17, std::io::Cursor::new(Vec::new()))
        };

        let empty_circuit = CircomCircuit {
            r1cs: config.r1cs,
            witness: Some(vec![]),
            wire_mapping: None,
            aux_offset: config.aux_offset,
        };

        let vk = halo2_proofs::plonk::keygen_vk(&params, &empty_circuit)?;
        let pk = halo2_proofs::plonk::keygen_pk(&params, vk.clone(), &empty_circuit)?;

        Ok(Self { circuit, pk, vk, params })
    }

    pub fn prove(&self, witness: Witness, pub_inputs: Vec<Fr>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut witness_map = witness.to_circom_input();
        let witness_vec = CircomCircuit::construct_witness_from_map(
            &self.circuit.r1cs,
            &mut witness_map,
            self.circuit.aux_offset,
        )?;

        let circuit_with_witness = CircomCircuit {
            r1cs: self.circuit.r1cs.clone(),
            witness: Some(witness_vec),
            wire_mapping: None,
            aux_offset: self.circuit.aux_offset,
        };

        // Mock verification
        let prover = MockProver::run(17, &circuit_with_witness, vec![pub_inputs.clone()])?;
        assert_eq!(prover.verify(), Ok(()), "Circuit constraints failed");

        // Real proof
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
}
