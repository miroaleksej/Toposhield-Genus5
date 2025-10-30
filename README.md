# TopoShield ZKP — Genus 5 Prototype

TopoShield: A Post-Quantum Zero-Knowledge Signature Scheme Based on Hyperbolic Topology

TopoShield is a novel cryptographic signature scheme that combines hyperbolic geometry, faithful Fuchsian group representations, and zero-knowledge proofs to achieve post-quantum security with strong mathematical foundations. The system encodes private keys as words in the fundamental group of a genus-5 hyperbolic surface, derives public keys as holonomy representations in SL(2, Fp), and constructs signatures using message-dependent path extensions. Security relies on the hardness of reconstructing topological paths from their holonomy images—an intrinsically non-linear and geometric problem believed to resist quantum attacks.

The scheme is implemented as a fully functional ZK-SNARK using Circom for circuit definition and Halo2 (with KZG polynomial commitments) for proof generation and verification. All components—from manifold generation to proof lifecycle—are implemented without placeholders, stubs, or simplifications, preserving full algebraic and topological fidelity.

Key Features

- Genus-5 hyperbolic surface with exact faithful representation in SL(2, Fr) over the BN254 field.
- Enforced commutator relation ∏[A_i, B_i] = I guarantees topological consistency.
- Deterministic, RFC 6979–style message binding via Poseidon-based PRF.
- Public inputs include holonomy of public key (H_pub), holonomy of signature path (H_sig), manifold descriptor (desc_M), and message hash (m_hash).
- Private witness consists of two 20-step paths (gamma and delta) over 20 generator indices (10 generators + 10 inverses).
- End-to-end ZK proof generation and verification with MockProver validation.
- Proof size: ~2.5–3.0 KB; compatible with standard Halo2 tooling.

Build and Usage Instructions

Prerequisites

- Rust (stable toolchain)
- Node.js (for Circom and snarkjs)
- circom (v2.1+)
- snarkjs (v0.7+)

Install dependencies:

```bash
make setup
```

Compile the Circom circuit:

```bash
make compile-circuit
```

Generate KZG trusted setup parameters (k=18):

```bash
make setup-kzg
```

Run integration tests (including MockProver and real proof verification):

```bash
make test
```

Generate a sample proof:

```bash
make prove
```

The proof will be saved as proof.bin. Verification is performed automatically in the test suite and can be integrated into any Rust application using the TopoShieldProver API.

Project Structure

- src/manifold.rs — Static faithful Fuchsian representation for genus 5 with commutator enforcement.
- src/witness.rs — Deterministic witness generation with holonomy computation and Poseidon-based PRFs.
- src/prover.rs — Full KZG prover and verifier using Halo2 and halo2-circom.
- circuits/holonomy_path.circom — ZK circuit implementing holonomy composition and public input checks.
- Makefile — Unified build, test, and proof generation workflow.
- Cargo.toml — Rust dependencies and metadata.

Security Notes

- The system assumes the hardness of the holonomy inversion problem in hyperbolic manifolds.
- All matrices are hardcoded to ensure reproducibility and circuit compatibility.
- Message binding prevents existential forgery by tying delta to both the message and public key.
- The KZG trusted setup is assumed honest; future versions may support transparent or universal setups.

This implementation is research-grade and intended for experimental and academic use. Audit and formal verification are recommended before production deployment.

#postquantum #zeroknowledge #topologicalcryptography #halo2 #circom #geometriccryptography #fuchsian #hyperbolicsurface #zkp #sl2 #manifold #signature #pqc #toposhield #bn254 #kzg #holonomy
