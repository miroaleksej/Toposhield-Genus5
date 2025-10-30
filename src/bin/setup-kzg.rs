// Minimal KZG setup generator
use std::fs;
use halo2_proofs::poly::kzg::commitment::ParamsKZG;
use halo2_proofs::halo2curves::bn256::Bn256;
use rand::rngs::OsRng;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all("params")?;
    let params = ParamsKZG::<Bn256>::setup(17, OsRng);
    let mut file = fs::File::create("params/kzg.srs")?;
    params.write(&mut file)?;
    println!("KZG trusted setup (k=17) generated.");
    Ok(())
}
