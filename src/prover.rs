// src/prover.rs
// TopoShield Prover: Halo2 + Circom integration for enhanced ZKP
// Compatible with holonomy_path_enhanced.circom (genus=5, path_len=20)

use crate::witness::Witness;
use ff::Field;
use halo2_circom::{
    circuit::{CircomCircuit, CircomConfig},
    plonk::CircomReduction,
};
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
            strategy::AccumulatorStrategy,
        },
    },
    transcript::{
        Blake2bRead, Blake2bWrite, Challenge255, TranscriptReadBuffer, TranscriptWriterBuffer,
    },
};
use std::{fs, io::Cursor};

pub struct TopoShieldProver {
    params: ParamsKZG<Bn256>,
    pk: ProvingKey<G1Affine>,
    vk: VerifyingKey<G1Affine>,
    r1cs: halo2_circom::circuit::R1CS<Bn256>,
    aux_offset: usize,
}

impl TopoShieldProver {
    /// Инициализирует прувера: загружает R1CS, WASM и KZG-параметры
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Загрузка Circom-артефактов
        let config = CircomConfig::<Bn256>::new(
            "build/holonomy_path_enhanced.r1cs",
            "build/holonomy_path_enhanced.wasm",
        )?;

        // Загрузка или генерация KZG SRS
        let params_path = "params/kzg.srs";
        let params = if std::path::Path::new(params_path).exists() {
            let bytes = fs::read(params_path)?;
            ParamsKZG::read::<_>(&mut Cursor::new(bytes))?
        } else {
            eprintln!("⚠️  KZG setup not found at params/kzg.srs — generating (k=17)...");
            let params = ParamsKZG::<Bn256>::setup(17, rand::rngs::OsRng);
            fs::create_dir_all("params")?;
            let mut file = fs::File::create(params_path)?;
            params.write(&mut file)?;
            params
        };

        // Пустая схема для генерации ключей
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

    /// Генерирует ZK-доказательство для заданного свидетельства
    pub fn prove(&self, witness: Witness) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Подготовка входов для Circom
        let mut witness_map = witness.to_circom_input();
        let witness_vec = CircomCircuit::construct_witness_from_map(
            &self.r1cs,
            &mut witness_map,
            self.aux_offset,
        )?;

        // Схема со свидетельством
        let circuit = CircomCircuit {
            r1cs: self.r1cs.clone(),
            witness: Some(witness_vec),
            wire_mapping: None,
            aux_offset: self.aux_offset,
        };

        // Публичные входы: H_pub, H_sig, desc_M, m_hash → 16 элементов
        let instances = vec![vec![
            witness.h_pub[0], witness.h_pub[1], witness.h_pub[2], witness.h_pub[3],
            witness.h_sig[0], witness.h_sig[1], witness.h_sig[2], witness.h_sig[3],
            witness.desc_m[0], witness.desc_m[1], witness.desc_m[2], witness.desc_m[3],
            witness.m_hash[0], witness.m_hash[1], witness.m_hash[2], witness.m_hash[3],
        ]];

        // Mock-верификация (для отладки)
        let mock_prover = MockProver::run(17, &circuit, instances.clone())?;
        assert_eq!(
            mock_prover.verify(),
            Ok(()),
            "Mock prover failed — check witness or circuit"
        );

        // Генерация реального доказательства
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

    /// Верифицирует доказательство
    pub fn verify(
        &self,
        proof: &[u8],
        h_pub: [Fr; 4],
        h_sig: [Fr; 4],
        desc_m: [Fr; 4],
        m_hash: [Fr; 4],
    ) -> Result<bool, Error> {
        let instances = vec![vec![
            h_pub[0], h_pub[1], h_pub[2], h_pub[3],
            h_sig[0], h_sig[1], h_sig[2], h_sig[3],
            desc_m[0], desc_m[1], desc_m[2], desc_m[3],
            m_hash[0], m_hash[1], m_hash[2], m_hash[3],
        ]];

        let strategy = AccumulatorStrategy::new(&self.params);
        let mut transcript = Blake2bRead::<_, G1Affine, Challenge255<_>>::init(proof);
        let result = verify_proof::<
            KZGCommitmentScheme<Bn256>,
            halo2_proofs::poly::kzg::multiopen::VerifierSHPLONK<_>,
            Challenge255<_>,
            AccumulatorStrategy<_>,
            _,
            Blake2bRead<_, _, _>,
        >(&self.params, &self.vk, strategy, &[instances.as_slice()], &mut transcript);

        match result {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}
