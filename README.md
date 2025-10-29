# TopoShield ZKP — Prototype for Genus 5

This repository contains a minimal working prototype of **TopoShield**, a topological cryptographic signature scheme based on the "Key with Topological Opening" concept. The implementation targets genus = 5 hyperbolic surfaces and uses zero-knowledge proofs to verify signatures without revealing the private path or manifold structure.

## Overview

The system models the private key as a word in the fundamental group π₁(ℳ) of a genus‑5 hyperbolic surface ℳ. The public key is the holonomy — a 2×2 matrix in SL(2, Fp) — obtained by composing faithful representations of the group generators. Signatures are derived by concatenating the private path with a message‑dependent path and computing the corresponding holonomy.

Verification is performed via a zk‑SNARK (Halo2 + Circom), which proves that:
- The public key and signature originate from the same manifold ℳ,
- Both are derived from the same private path γ,
- No information about γ, the manifold geometry, or internal structure is disclosed.

The design strictly adheres to the four axioms from the theoretical foundation:
- **DT (Discrete Torus)**: Parameter space emulates toroidal topology via SL(2, Fp) matrices.
- **S (Strata)**: Paths correspond to strata defined by the secret key.
- **Sym (Symmetry)**: ZKP preserves the (Uᵣ, U_z) ↔ (−Uᵣ, −U_z) symmetry.
- **E (Ergodicity)**: Path generation ensures ergodic traversal of the space.

## Features

- Faithful Fuchsian representation of π₁(ℳ) for genus = 5
- Path-based private keys as sequences of generator indices (0–19)
- Holonomy computation via sequential SL(2, Fp) matrix multiplication
- Circom circuit for ZK verification of signature consistency
- Rust backend using halo2_circom for proof generation and verification
- Deterministic path derivation from message and private key

## Requirements

- Rust 1.70 or later
- Node.js (for Circom)
- Linux or macOS

## Quick Start

1. Clone the repository:
   ```
   git clone https://github.com/yourname/toposhield-genus5
   cd toposhield-genus5
   ```

2. Install dependencies:
   ```
   make setup
   ```

3. Compile the Circom circuit:
   ```
   make compile-circuit
   ```

4. Generate KZG trusted setup parameters (one-time):
   ```
   make setup-kzg
   ```

5. Run the end-to-end test (generates key, signs, proves, verifies):
   ```
   make test
   ```

## Limitations

- This is a **prototype** for research and demonstration purposes.
- Only genus = 5 is supported; higher genera require parameter re-tuning.
- The faithful representation uses hardcoded matrices verified offline.
- Performance is not optimized for production use.

topological-cryptography  
zero-knowledge-proofs  
zkp  
halo2  
circom  
post-quantum-cryptography  
algebraic-topology  
computational-topology  
tropical-security  
ecdsa-audit  
persistent-homology  
bettinumbers  
hyperbolic-geometry  
fundamental-group  
holonomy  
topological-data-analysis  
zk-snark  
bls12-381  
poseidon-hash  
cryptographic-security
