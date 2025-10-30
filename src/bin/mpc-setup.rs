// src/bin/mpc-setup.rs
// Multi-Party Computation for KZG Trusted Setup
// Implements a secure distributed generation of KZG SRS

use std::fs;
use std::io::{self, Cursor, Read, Write};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use halo2_proofs::poly::kzg::commitment::ParamsKZG;
use halo2_proofs::halo2curves::bn256::Bn256;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

/// Configuration for MPC Trusted Setup
#[derive(Serialize, Deserialize, Clone)]
struct MpcConfig {
    /// Security parameter (k value for KZG)
    k: u32,
    /// Total number of participants
    total_participants: u32,
    /// Minimum number of participants required to finalize
    threshold: u32,
    /// Session identifier
    session_id: String,
    /// Base directory for session files
    base_dir: String,
}

impl MpcConfig {
    /// Creates a new MPC configuration
    fn new(k: u32, total_participants: u32, threshold: u32) -> Self {
        let session_id = generate_session_id();
        let base_dir = format!("mpc-setup-{}", session_id);
        
        Self {
            k,
            total_participants,
            threshold,
            session_id,
            base_dir,
        }
    }
    
    /// Gets the path for participant file
    fn participant_path(&self, participant_id: u32) -> String {
        format!("{}/participant-{}.bin", self.base_dir, participant_id)
    }
    
    /// Gets the path for final SRS file
    fn srs_path(&self) -> String {
        format!("{}/kzg.srs", self.base_dir)
    }
    
    /// Gets the path for SRS hash file
    fn srs_hash_path(&self) -> String {
        format!("{}/kzg.srs.sha256", self.base_dir)
    }
}

/// MPC Trusted Setup session
struct MpcSession {
    config: MpcConfig,
    participant_id: u32,
    current_round: u32,
    total_rounds: u32,
}

impl MpcSession {
    /// Creates a new MPC session or joins an existing one
    fn new(config: MpcConfig, participant_id: u32) -> io::Result<Self> {
        // Create base directory if it doesn't exist
        fs::create_dir_all(&config.base_dir)?;
        
        // Initialize session state
        let current_round = if participant_id == 1 {
            1
        } else {
            // Check if previous participant has completed their round
            let prev_path = config.participant_path(participant_id - 1);
            if Path::new(&prev_path).exists() {
                participant_id
            } else {
                participant_id - 1
            }
        };
        
        // Total rounds equals total participants
        let total_rounds = config.total_participants;
        
        Ok(Self {
            config,
            participant_id,
            current_round,
            total_rounds,
        })
    }
    
    /// Initializes the MPC setup process
    fn initialize(&mut self) -> io::Result<()> {
        if self.participant_id != 1 {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Only participant 1 can initialize the setup",
            ));
        }
        
        // Generate initial parameters
        let params = ParamsKZG::<Bn256>::setup(self.config.k, OsRng);
        
        // Save initial contribution
        self.save_contribution(&params)?;
        
        Ok(())
    }
    
    /// Executes the current round of MPC
    fn execute_round(&mut self) -> io::Result<()> {
        if self.current_round != self.participant_id {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Cannot execute round {} as participant {}",
                    self.current_round, self.participant_id
                ),
            ));
        }
        
        // Load previous contribution
        let prev_params = if self.participant_id > 1 {
            self.load_previous_contribution()?
        } else {
            // First participant uses fresh parameters
            ParamsKZG::<Bn256>::setup(self.config.k, OsRng)
        };
        
        // Generate random delta
        let delta = Fr::random(OsRng);
        
        // Apply contribution
        let mut params = prev_params;
        params.contribute(delta, self.config.k)?;
        
        // Save contribution
        self.save_contribution(&params)?;
        
        // Move to next round
        self.current_round += 1;
        
        Ok(())
    }
    
    /// Finalizes the MPC setup and generates the final SRS
    fn finalize(&mut self) -> io::Result<ParamsKZG<Bn256>> {
        if self.participant_id != self.config.total_participants {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Only the last participant can finalize the setup",
            ));
        }
        
        // Load the final contribution
        let params = self.load_previous_contribution()?;
        
        // Save the final SRS
        let srs_path = self.config.srs_path();
        let mut file = fs::File::create(&srs_path)?;
        params.write(&mut file)?;
        
        // Compute and save hash for integrity verification
        let hash = calculate_file_hash(&srs_path);
        fs::write(self.config.srs_hash_path(), hash)?;
        
        Ok(params)
    }
    
    /// Saves a contribution to disk
    fn save_contribution(&self, params: &ParamsKZG<Bn256>) -> io::Result<()> {
        let path = self.config.participant_path(self.participant_id);
        let mut file = fs::File::create(path)?;
        params.write(&mut file)?;
        Ok(())
    }
    
    /// Loads the previous contribution from disk
    fn load_previous_contribution(&self) -> io::Result<ParamsKZG<Bn256>> {
        let prev_id = self.participant_id - 1;
        let path = self.config.participant_path(prev_id);
        
        let mut file = fs::File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        
        ParamsKZG::read(&mut Cursor::new(buffer))
    }
    
    /// Checks if the participant is the first in the session
    fn is_first_participant(&self) -> bool {
        self.participant_id == 1
    }
    
    /// Checks if the participant is the last in the session
    fn is_last_participant(&self) -> bool {
        self.participant_id == self.config.total_participants
    }
    
    /// Gets the current progress of the MPC setup
    fn get_progress(&self) -> (u32, u32) {
        (self.current_round - 1, self.total_rounds)
    }
}

/// Generates a unique session ID
fn generate_session_id() -> String {
    use rand::Rng;
    let mut rng = OsRng;
    let id: u64 = rng.gen();
    format!("{:x}", id)
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
    let total_participants = 5;
    let threshold = 3;
    
    // Get participant ID from command line or environment
    let participant_id = match std::env::args().nth(1) {
        Some(id) => id.parse().expect("Invalid participant ID"),
        None => 1, // Default to first participant
    };
    
    println!("MPC Trusted Setup for TopoShield (k={})", k);
    println!("  Participant ID: {}", participant_id);
    println!("  Total participants: {}", total_participants);
    println!("  Threshold: {}", threshold);
    
    // Create or join MPC session
    let mut config = MpcConfig::new(k, total_participants, threshold);
    let mut session = MpcSession::new(config, participant_id)?;
    
    // Get progress
    let (completed, total) = session.get_progress();
    println!("  Progress: {} of {} rounds completed", completed, total);
    
    // Initialize if first participant
    if session.is_first_participant() {
        println!("  Initializing MPC setup process...");
        session.initialize()?;
        println!("  Initialization complete. Waiting for next participant...");
    }
    
    // Execute current round
    println!("  Executing round {}...", session.current_round);
    session.execute_round()?;
    println!("  Round {} completed.", session.current_round - 1);
    
    // Finalize if last participant
    if session.is_last_participant() {
        println!("  Finalizing MPC setup and generating SRS...");
        session.finalize()?;
        println!("  MPC Trusted Setup completed successfully.");
        println!("  SRS saved to {}", session.config.srs_path());
        println!("  SHA-256 hash: {:x}", calculate_file_hash(session.config.srs_path()));
    } else {
        println!("  Please notify participant {} to continue the setup.", 
                 participant_id + 1);
    }
    
    Ok(())
}
