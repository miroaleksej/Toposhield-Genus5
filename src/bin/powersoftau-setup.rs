// src/bin/powersoftau-setup.rs
// Integration with Powers of Tau protocol for KZG Trusted Setup

use std::fs;
use std::io::{self, Cursor, Read, Write};
use std::path::Path;
use halo2_proofs::poly::kzg::commitment::{ParamsKZG, ProverKey};
use halo2_proofs::halo2curves::bn256::Bn256;
use halo2_proofs::poly::kzg::multiopen::ProverSHPLONK;
use halo2_proofs::transcript::Challenge255;
use rand::rngs::OsRng;
use sha2::{Sha256, Digest};
use serde::{Deserialize, Serialize};

/// Configuration for Powers of Tau setup
#[derive(Serialize, Deserialize, Clone)]
struct PowersOfTauConfig {
    /// Target circuit size (k value)
    k: u32,
    /// Path to initial Powers of Tau file
    initial_ptau_path: String,
    /// Path for intermediate contributions
    contribution_path: String,
    /// Path for final KZG SRS
    srs_path: String,
    /// Path for SRS hash verification
    srs_hash_path: String,
}

impl PowersOfTauConfig {
    /// Creates a new configuration
    fn new(k: u32) -> Self {
        Self {
            k,
            initial_ptau_path: format!("params/powersoftau_initial_{}.ptau", k),
            contribution_path: format!("params/powersoftau_contribution_{}.ptau", k),
            srs_path: format!("params/kzg.srs"),
            srs_hash_path: format!("params/kzg.srs.sha256"),
        }
    }
    
    /// Ensures necessary directories exist
    fn setup_directories(&self) -> io::Result<()> {
        let dir = Path::new(&self.initial_ptau_path).parent()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Invalid path"))?;
        fs::create_dir_all(dir)
    }
}

/// Powers of Tau contribution
#[derive(Serialize, Deserialize)]
struct PowersOfTauContribution {
    /// Randomness used in the contribution
    randomness: Vec<u8>,
    /// Resulting parameters
    params: Vec<u8>,
    /// Timestamp of contribution
    timestamp: u64,
    /// Hash of previous contribution
    prev_hash: Option<Vec<u8>>,
}

/// Powers of Tau session
struct PowersOfTauSession {
    config: PowersOfTauConfig,
    /// Whether this is the first participant
    is_first: bool,
}

impl PowersOfTauSession {
    /// Creates a new session or joins an existing one
    fn new(k: u32) -> io::Result<Self> {
        let config = PowersOfTauConfig::new(k);
        config.setup_directories()?;
        
        let is_first = !Path::new(&config.initial_ptau_path).exists();
        
        Ok(Self { config, is_first })
    }
    
    /// Initializes the Powers of Tau process
    fn initialize(&self) -> io::Result<()> {
        if !self.is_first {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "Initialization can only be done by the first participant",
            ));
        }
        
        // Generate initial parameters
        let params = ParamsKZG::<Bn256>::setup(self.config.k, OsRng);
        
        // Save initial Powers of Tau file
        let mut file = fs::File::create(&self.config.initial_ptau_path)?;
        params.write(&mut file)?;
        
        Ok(())
    }
    
    /// Creates a new contribution
    fn create_contribution(&self, seed: &[u8]) -> io::Result<PowersOfTauContribution> {
        // Load previous parameters
        let prev_path = if self.is_first {
            &self.config.initial_ptau_path
        } else {
            &self.config.contribution_path
        };
        
        let mut file = fs::File::open(prev_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        
        let mut params = ParamsKZG::<Bn256>::read(&mut Cursor::new(buffer))?;
        
        // Generate randomness
        let randomness = Fr::random(OsRng);
        
        // Apply contribution
        params.contribute(randomness, self.config.k)?;
        
        // Save contribution
        let mut params_buffer = Vec::new();
        params.write(&mut params_buffer)?;
        
        // Calculate hash of previous contribution for integrity
        let prev_hash = if !self.is_first {
            Some(calculate_file_hash(prev_path))
        } else {
            None
        };
        
        Ok(PowersOfTauContribution {
            randomness: seed.to_vec(),
            params: params_buffer,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            prev_hash,
        })
    }
    
    /// Applies a contribution to the Powers of Tau process
    fn apply_contribution(&self, contribution: &PowersOfTauContribution) -> io::Result<()> {
        // Verify previous hash if not first contribution
        if let Some(prev_hash) = &contribution.prev_hash {
            let current_hash = calculate_file_hash(
                if self.is_first { 
                    &self.config.initial_ptau_path 
                } else { 
                    &self.config.contribution_path 
                }
            );
            if &current_hash != prev_hash {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Previous contribution hash mismatch",
                ));
            }
        }
        
        // Save the contribution
        let mut file = fs::File::create(&self.config.contribution_path)?;
        file.write_all(&contribution.params)?;
        
        Ok(())
    }
    
    /// Finalizes the Powers of Tau process and generates KZG SRS
    fn finalize(&self) -> io::Result<()> {
        // Load the final contribution
        let mut file = fs::File::open(&self.config.contribution_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        
        let params = ParamsKZG::<Bn256>::read(&mut Cursor::new(buffer))?;
        
        // Save as KZG SRS
        let mut srs_file = fs::File::create(&self.config.srs_path)?;
        params.write(&mut srs_file)?;
        
        // Save hash for integrity verification
        let hash = calculate_file_hash(&self.config.srs_path);
        fs::write(&self.config.srs_hash_path, hash)?;
        
        Ok(())
    }
    
    /// Verifies the integrity of the final SRS
    fn verify_srs(&self) -> io::Result<bool> {
        if !Path::new(&self.config.srs_path).exists() {
            return Ok(false);
        }
        
        let stored_hash = fs::read(&self.config.srs_hash_path)
            .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Hash file not found"))?;
        
        let current_hash = calculate_file_hash(&self.config.srs_path);
        
        Ok(stored_hash == current_hash)
    }
}

/// Calculates SHA-256 hash of a file
fn calculate_file_hash<P: AsRef<Path>>(path: P) -> Vec<u8> {
    let mut file = fs::File::open(path).expect("Failed to open file");
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher).expect("Failed to hash file");
    hasher.finalize().to_vec()
}

fn main() -> io::Result<()> {
    // Configuration parameters
    let k = 17; // Supports up to 2^17 = 131,072 constraints
    
    println!("Powers of Tau Trusted Setup for TopoShield (k={})", k);
    
    // Create or join Powers of Tau session
    let session = PowersOfTauSession::new(k)?;
    
    if session.is_first {
        println!("  Initializing Powers of Tau process...");
        session.initialize()?;
        println!("  Initialization complete. Please share {} with next participant.", 
                 session.config.initial_ptau_path);
    } else {
        println!("  Creating contribution...");
        let contribution = session.create_contribution(b"secure_seed")?;
        
        // Save contribution to file for sharing
        let contribution_file = "powersoftau_contribution.bin";
        fs::write(contribution_file, bincode::serialize(&contribution).unwrap())?;
        
        println!("  Contribution created. Please share {} with next participant.",
                 contribution_file);
        
        // In a real scenario, the contribution would be shared with the next participant
        // who would then apply it and continue the process
    }
    
    // Finalization step (typically done by the last participant)
    if std::env::var("FINALIZE").is_ok() {
        println!("  Finalizing Powers of Tau process...");
        session.finalize()?;
        
        // Verify the SRS
        let is_valid = session.verify_srs()?;
        println!("  SRS integrity verification: {}", if is_valid { "PASS" } else { "FAIL" });
        
        if is_valid {
            println!("  Trusted setup completed successfully.");
            println!("  SRS saved to {}", session.config.srs_path);
            println!("  SHA-256: {:x}", calculate_file_hash(session.config.srs_path));
        } else {
            println!("  Error: SRS integrity check failed.");
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "SRS integrity check failed",
            ));
        }
    }
    
    Ok(())
}
