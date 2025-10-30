# TopoShield ZKP — Genus 5 Prototype
# Makefile for building, testing, and proving

# Directories
CIRCUIT_DIR := circuits
BUILD_DIR := build
PARAMS_DIR := params
SRC_DIR := src

# Artifacts
CIRCUIT := $(CIRCUIT_DIR)/holonomy_path.circom
R1CS := $(BUILD_DIR)/holonomy_path.r1cs
WASM := $(BUILD_DIR)/holonomy_path.wasm
SRS := $(PARAMS_DIR)/kzg.srs

# Tools
CIRCOM := circom
SNARKJS := snarkjs
RUSTC := rustc
CARGO := cargo

# Default target
.PHONY: all
all: setup compile-circuit setup-kzg test

# Install system and Node.js dependencies
.PHONY: setup
setup:
	@echo "🔧 Installing Rust dependencies..."
	$(CARGO) build --release
	@echo "📦 Installing Circom and snarkjs (if not present)..."
	@which $(CIRCOM) > /dev/null || (echo "❌ circom not found. Please install: npm install -g circom" && exit 1)
	@which $(SNARKJS) > /dev/null || (echo "❌ snarkjs not found. Please install: npm install -g snarkjs" && exit 1)
	@echo "✅ Setup complete."

# Compile Circom circuit
$(BUILD_DIR):
	@mkdir -p $@

$(R1CS) $(WASM): $(CIRCUIT) | $(BUILD_DIR)
	@echo "⚙️  Compiling Circom circuit..."
	$(CIRCOM) $< --r1cs --wasm --sym --output $(BUILD_DIR)

.PHONY: compile-circuit
compile-circuit: $(R1CS) $(WASM)
	@echo "✅ Circuit compiled to $(BUILD_DIR)/"

# Generate KZG trusted setup (one-time)
$(PARAMS_DIR):
	@mkdir -p $@

$(SRS): | $(PARAMS_DIR)
	@echo "🔐 Generating KZG trusted setup (k=17)..."
	$(CARGO) run --bin setup-kzg --release
	@echo "✅ KZG parameters saved to $(SRS)"

.PHONY: setup-kzg
setup-kzg: $(SRS)

# Run Rust integration tests
.PHONY: test
test:
	@echo "🧪 Running integration tests..."
	$(CARGO) test --release
	@echo "✅ All tests passed."

# Generate a sample ZK proof
.PHONY: prove
prove:
	@echo "🧾 Generating ZK proof..."
	$(CARGO) run --bin prove-example --release
	@echo "✅ Proof generated."

# Clean build artifacts
.PHONY: clean
clean:
	@echo "🧹 Cleaning build artifacts..."
	rm -rf $(BUILD_DIR) $(PARAMS_DIR)
	$(CARGO) clean
	@echo "✅ Cleaned."

# Rebuild everything
.PHONY: rebuild
rebuild: clean all

# Show project info
.PHONY: info
info:
	@echo "TopoShield ZKP — Genus 5 Prototype"
	@echo "----------------------------------"
	@echo "Private key = path γ in π₁(ℳ)"
	@echo "Public key  = Hol(γ) ∈ SL(2, Fp)"
	@echo "Signature   = Hol(γ · δ(m))"
	@echo "Verification = Halo2 + Circom ZKP"
	@echo ""
	@echo "Quick Start:"
	@echo "  make setup          # Install dependencies"
	@echo "  make compile-circuit # Compile Circom"
	@echo "  make setup-kzg      # Generate KZG setup"
	@echo "  make test           # Run tests"
	@echo "  make prove          # Generate proof"
