.PHONY: build test run clean release install

# Default target
all: build

# Build the project
build:
	@echo "Building QitOps Agent..."
	cargo build

# Run tests
test:
	@echo "Running tests..."
	cargo test

# Run the application
run:
	@echo "Running QitOps Agent..."
	cargo run -- $(ARGS)

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean

# Build release version
release:
	@echo "Building release version..."
	cargo build --release

# Install the application
install: release
	@echo "Installing QitOps Agent..."
	cargo install --path .

# Run with specific commands
llm-list:
	@echo "Listing LLM providers..."
	cargo run -- llm list

llm-test:
	@echo "Testing LLM provider..."
	cargo run -- llm test -t "What is software testing?"

# Help
help:
	@echo "QitOps Agent Makefile"
	@echo ""
	@echo "Usage:"
	@echo "  make build     - Build the project"
	@echo "  make test      - Run tests"
	@echo "  make run       - Run the application (use ARGS=\"llm list\" to pass arguments)"
	@echo "  make clean     - Clean build artifacts"
	@echo "  make release   - Build release version"
	@echo "  make install   - Install the application"
	@echo "  make llm-list  - List LLM providers"
	@echo "  make llm-test  - Test LLM provider"
	@echo "  make help      - Show this help message"
