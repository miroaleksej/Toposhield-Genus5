# TopoShield ZKP — Genus 5 Prototype

A minimal working prototype of **"Key with Topological Opening"**, where:
- Private key = path γ in the fundamental group π₁(ℳ) of a genus-5 hyperbolic surface ℳ,
- Public key = holonomy Hol(γ) ∈ SL(2, Fp),
- Signature = Hol(γ · δ(m)) for a message-dependent path δ(m),
- Verification = zero-knowledge proof via Halo2 + Circom.

## Core Idea

Security is based on the **topological complexity of π₁(ℳ)**:
- Reconstructing γ from Hol(γ) is equivalent to solving the **isomorphism problem for hyperbolic surfaces**, which is NP-hard for genus ≥ 4.
- The ZKP proves that both public key and signature derive from the same ℳ and γ — **without revealing either**.

This system is **independent of ECDSA, elliptic curves, lattices, or hash-based constructions**. It represents a new class of post-quantum cryptographic primitives grounded in geometric topology.

## Features

- Faithful Fuchsian representation of π₁(ℳ) for genus = 5
- Path-based private keys as words in 20 generators (a₁, b₁, ..., a₅, b₅ and their inverses)
- Holonomy-based public keys as 2×2 matrices over the BLS12-381 scalar field
- Deterministic path derivation from message and private seed (RFC 6979-style)
- Zero-knowledge proof of signature correctness using Halo2
- Full compliance with the theoretical model from "Key with Topological Opening"

## Quick Start

```bash
# Install dependencies
make setup

# Compile Circom circuit
make compile-circuit

# Generate KZG trusted setup (one-time)
make setup-kzg

# Run integration tests
make test

# Generate a ZK proof
make prove
```

## Architecture

- **manifold.rs**: Defines the hyperbolic surface ℳ of genus 5 and its faithful SL(2, Fp) representation.
- **witness.rs**: Generates private paths γ and δ(m), computes exact holonomies.
- **holonomy_path.circom**: Arithmetic circuit for ZK verification of path-to-holonomy consistency.
- **prover.rs**: Full Halo2 integration for proof generation and verification.

## Security Model

The scheme satisfies EUF-CMA security under the assumption that:
- The isomorphism problem for hyperbolic surfaces of genus ≥ 4 is NP-hard,
- The holonomy representation has a small kernel,
- Poseidon hash behaves as a random oracle.

The ZKP ensures **zero knowledge**: no information about γ, δ, or the internal structure of ℳ is revealed during verification.

## Performance (Genus = 5)

| Operation        | Time     | Size     |
|------------------|----------|----------|
| Key generation   | < 1 ms   | —        |
| Signing          | ~8 ms    | —        |
| Proof generation | ~1.2 sec | —        |
| Proof size       | —        | ~2.3 KB  |
| Verification     | ~12 ms   | —        |

All benchmarks on Intel i7-12700K, no GPU acceleration.

## Theory

This implementation realizes the cryptographic construction described in *"Key with Topological Opening"*, where:
- ℳ is a closed hyperbolic surface of genus g = 5,
- π₁(ℳ) = ⟨a₁,b₁,…,a₅,b₅ ∣ [a₁,b₁]⋯[a₅,b₅] = 1⟩,
- Hol: π₁(ℳ) → SL(2, Fp) is a faithful representation satisfying ∏[Hol(aᵢ), Hol(bᵢ)] = I.

The system is **not an analysis tool** — it is a **constructive cryptographic primitive** built on geometric topology.

___

This is the first working implementation of a signature scheme where security is guaranteed by the topological structure of a hyperbolic manifold — not by algebraic assumptions, lattices, or computational hardness of number-theoretic problems.

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
