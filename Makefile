# TopoShield ZKP - Genus 5 Prototype
# Makefile for building, testing, and proving with enhanced security features

# Directories
CIRCUIT_DIR := circuits
BUILD_DIR := build
PARAMS_DIR := params
SRC_DIR := src

# Artifacts
CIRCUIT := $(CIRCUIT_DIR)/holonomy_path_enhanced.circom
R1CS := $(BUILD_DIR)/holonomy_path_enhanced.r1cs
WASM := $(BUILD_DIR)/holonomy_path_enhanced.wasm
SRS := $(PARAMS_DIR)/kzg.srs
SRS_HASH := $(PARAMS_DIR)/kzg.srs.sha256

# Tools
CIRCOM := circom
RUSTC := rustc
CARGO := cargo

# Default target
.PHONY: all
all: setup compile-circuit setup-kzg test

# Install system and Node.js dependencies
.PHONY: setup
setup:
	@echo "Installing Rust dependencies..."
	$(CARGO) build --release
	@echo "Checking Circom installation..."
	@which $(CIRCOM) > /dev/null || (echo "circom not found. Please install: npm install -g circom" && exit 1)
	@echo "Setup complete."

# Create build directory
$(BUILD_DIR):
	@mkdir -p $@

# Compile Circom circuit
$(R1CS) $(WASM): $(CIRCUIT) | $(BUILD_DIR)
	@echo "Compiling enhanced Circom circuit..."
	$(CIRCOM) $< --r1cs --wasm --sym --output $(BUILD_DIR)

.PHONY: compile-circuit
compile-circuit: $(R1CS) $(WASM)
	@echo "Circuit compiled to $(BUILD_DIR)/"

# Create params directory
$(PARAMS_DIR):
	@mkdir -p $@

# MPC Trusted Setup
.PHONY: mpc-setup
mpc-setup:
	@echo "Starting MPC Trusted Setup (requires multiple participants)..."
	$(CARGO) run --bin mpc-setup --release
	@echo "MPC Trusted Setup completed."

# Powers of Tau Trusted Setup
.PHONY: powersoftau-setup
powersoftau-setup:
	@echo "Starting Powers of Tau Trusted Setup..."
	$(CARGO) run --bin powersoftau-setup --release
	@echo "Powers of Tau setup completed."

# Verify SRS integrity
.PHONY: verify-srs
verify-srs:
	@echo "Verifying KZG SRS integrity..."
	$(CARGO) run --bin verify-srs --release
	@echo "SRS integrity verification completed."

# Generate KZG trusted setup (local, for development only)
.PHONY: setup-kzg
setup-kzg:
	@echo "WARNING: For production use 'make mpc-setup' instead of local generation!"
	@echo "Generating local KZG trusted setup (k=17)..."
	$(CARGO) run --bin setup-kzg --release
	@echo "Local KZG trusted setup saved to $(SRS)"
	@echo "Generating SRS hash for integrity verification..."
	sha256sum $(SRS) > $(SRS_HASH)
	@echo "SRS hash saved to $(SRS_HASH)"

# Run Rust integration tests
.PHONY: test
test:
	@echo "Running integration tests..."
	$(CARGO) test --release
	@echo "All tests passed."

# Generate a sample ZK proof
.PHONY: prove
prove:
	@echo "Generating ZK proof with enhanced validation..."
	$(CARGO) run --bin prove-example --release
	@echo "Proof generated."

# Clean build artifacts
.PHONY: clean
clean:
	@echo "Cleaning build artifacts..."
	rm -rf $(BUILD_DIR) $(PARAMS_DIR) proof.bin
	$(CARGO) clean
	@echo "Cleaned."

# Rebuild everything
.PHONY: rebuild
rebuild: clean all

# Show project information
.PHONY: info
info:
	@echo "TopoShield ZKP - Genus 5 Prototype"
	@echo "---------------------------------"
	@echo "Private key = reduced path γ in π₁(ℳ)"
	@echo "Public key = Hol(γ) ∈ SL(2, Fp)"
	@echo "Signature = Hol(γ · δ(m))"
	@echo "Verification = Halo2 + Circom ZKP with structural checks"
	@echo ""
	@echo "Quick Start:"
	@echo "  make setup          # Install dependencies"
	@echo "  make compile-circuit # Compile enhanced Circom"
	@echo "  make mpc-setup      # Generate MPC trusted setup (recommended)"
	@echo "  OR"
	@echo "  make powersoftau-setup # Generate trusted setup using Powers of Tau"
	@echo "  make test           # Run tests"
	@echo "  make prove          # Generate proof"
