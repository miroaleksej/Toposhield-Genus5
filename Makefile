# TopoShield ZKP — Makefile for genus=5 prototype
# Fully automated build, test, and proof generation

# Configuration
CIRCUIT_NAME = holonomy_path
CIRCUIT_DIR = circuits
BUILD_DIR = build
PARAMS_DIR = params
SRC_DIR = src
TESTS_DIR = tests

# Tools
CIRCOM = circom
RUSTC = cargo
SNARKJS = npx snarkjs

# Halo2 parameters (k=17 supports ~131k constraints)
K_PARAM = 17

# Default target
.PHONY: all
all: setup compile-circuit setup-kzg test

# Install dependencies
.PHONY: setup
setup:
	@echo "[+] Installing dependencies..."
	@which $(CIRCOM) > /dev/null || npm install -g circom
	@$(RUSTC) --version > /dev/null || (echo "Error: Rust not found. Install Rust: https://rustup.rs" && exit 1)
	@$(RUSTC) build --quiet
	@echo "[+] Dependencies installed."

# Compile Circom circuit
.PHONY: compile-circuit
compile-circuit: setup
	@echo "[+] Compiling Circom circuit..."
	@mkdir -p $(BUILD_DIR)
	$(CIRCOM) $(CIRCUIT_DIR)/$(CIRCUIT_NAME).circom \
		--r1cs \
		--wasm \
		--sym \
		--output $(BUILD_DIR)/
	@echo "[+] Circuit compiled: $(BUILD_DIR)/$(CIRCUIT_NAME).r1cs"

# Generate KZG trusted setup parameters (one-time)
.PHONY: setup-kzg
setup-kzg: compile-circuit
	@echo "[+] Generating KZG parameters (k=$(K_PARAM))..."
	@mkdir -p $(PARAMS_DIR)
	@if [ ! -f "$(PARAMS_DIR)/kzg.srs" ]; then \
		$(RUSTC) run --example setup_kzg --release -- --k=$(K_PARAM) --output=$(PARAMS_DIR)/kzg.srs; \
	else \
		echo "[+] KZG parameters already exist."; \
	fi

# Run integration tests
.PHONY: test
test: setup-kzg
	@echo "[+] Running integration tests..."
	$(RUSTC) test --release

# Generate a proof for a test message
.PHONY: prove
prove: test
	@echo "[+] Generating ZK proof..."
	$(RUSTC) run --release --bin prover_example

# Clean build artifacts
.PHONY: clean
clean:
	@echo "[-] Cleaning build artifacts..."
	rm -rf $(BUILD_DIR)/*
	rm -rf $(PARAMS_DIR)/*
	cargo clean

# Rebuild everything
.PHONY: rebuild
rebuild: clean all

# Verify Circom circuit constraints (debug)
.PHONY: debug-circuit
debug-circuit: compile-circuit
	@echo "[+] Debugging circuit constraints..."
	$(CIRCOM) $(CIRCUIT_DIR)/$(CIRCUIT_NAME).circom --r1cs --sym --wat --output $(BUILD_DIR)/
	@echo "[+] WAT file generated for debugging."

# Help
.PHONY: help
help:
	@echo "TopoShield ZKP Makefile — genus=5 prototype"
	@echo ""
	@echo "Targets:"
	@echo "  setup          Install dependencies"
	@echo "  compile-circuit Compile Circom circuit"
	@echo "  setup-kzg      Generate KZG trusted setup (one-time)"
	@echo "  test           Run integration tests"
	@echo "  prove          Generate a ZK proof"
	@echo "  clean          Remove build artifacts"
	@echo "  rebuild        Clean and rebuild everything"
	@echo "  debug-circuit  Generate WAT for constraint debugging"
	@echo ""
	@echo "Quick start: make setup && make prove"
