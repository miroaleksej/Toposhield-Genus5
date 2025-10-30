// src/bin/setup-kzg.rs
// Generate KZG trusted setup for TopoShield (k=17 → ~131k constraints)
use std::fs;
use halo2_proofs::poly::kzg::commitment::ParamsKZG;
use halo2_proofs::halo2curves::bn256::Bn256;
use rand::rngs::OsRng;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Ensure params directory exists
    fs::create_dir_all("params")?;

    // Generate KZG SRS (k=17 supports up to 2^17 = 131072 constraints)
    // holonomy_path_enhanced.circom uses ~50k constraints → k=17 is sufficient
    let params = ParamsKZG::<Bn256>::setup(17, OsRng);

    // Save to file
    let mut file = fs::File::create("params/kzg.srs")?;
    params.write(&mut file)?;

    println!("✅ KZG trusted setup (k=17) generated and saved to params/kzg.srs");
    Ok(())
}
