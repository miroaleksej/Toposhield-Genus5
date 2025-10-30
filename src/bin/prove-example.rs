// Example proof generation
use toposhield::{witness::Witness, prover::TopoShieldProver};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let prover = TopoShieldProver::new()?;
    let witness = Witness::new(b"TopoShield proof example", b"example_seed");
    let proof = prover.prove(witness)?;
    fs::write("proof.bin", &proof)?;
    println!("Proof saved to proof.bin ({} bytes)", proof.len());
    Ok(())
}
