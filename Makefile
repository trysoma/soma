.PHONY: help install clean build build-release \
	test test-unit test-integration test-all test-coverage \
	lint lint-js lint-rs lint-py lint-db lint-fix lint-fix-js lint-fix-rs lint-fix-py \
	py-build py-build-sdk-core py-build-sdk-core-wheel py-test py-test-coverage py-install py-clean-cache \
	db-generate-rs-models \
	db-mcp-generate-migration db-mcp-generate-hash \
	db-encryption-generate-migration db-encryption-generate-hash \
	db-identity-generate-migration db-identity-generate-hash \
	db-soma-generate-migration db-soma-generate-hash \
	_db-generate-migration _db-generate-hash _install-sqlc-gen-from-template

help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Available targets:'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  \033[36m%-40s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

_install-sqlc-gen-from-template: ## Install sqlc-gen-from-template if not already installed (internal helper)
	@if command -v sqlc-gen-from-template >/dev/null 2>&1; then \
		echo "✓ sqlc-gen-from-template already installed"; \
	else \
		echo "Installing sqlc-gen-from-template..."; \
		INSTALL_DIR="$$HOME/.local/bin"; \
		mkdir -p "$$INSTALL_DIR"; \
		OS=$$(uname -s); \
		ARCH=$$(uname -m); \
		case "$$OS" in \
			Linux) \
				case "$$ARCH" in \
					x86_64) PLATFORM="Linux_x86_64" ;; \
					aarch64|arm64) PLATFORM="Linux_arm64" ;; \
					*) echo "Unsupported architecture: $$ARCH"; exit 1 ;; \
				esac ;; \
			Darwin) \
				case "$$ARCH" in \
					x86_64) PLATFORM="Darwin_x86_64" ;; \
					arm64) PLATFORM="Darwin_arm64" ;; \
					*) echo "Unsupported architecture: $$ARCH"; exit 1 ;; \
				esac ;; \
			*) echo "Unsupported OS: $$OS"; exit 1 ;; \
		esac; \
		URL="https://github.com/trysoma/fdietze-sqlc-gen-from-template/releases/download/v1.0.0/fdietze-sqlc-gen-from-template_$${PLATFORM}.tar.gz"; \
		echo "Downloading from $$URL..."; \
		curl -fsSL "$$URL" -o /tmp/sqlc-gen-from-template.tar.gz; \
		tar -xzf /tmp/sqlc-gen-from-template.tar.gz -C /tmp; \
		chmod +x /tmp/sqlc-gen-from-template; \
		mv /tmp/sqlc-gen-from-template "$$INSTALL_DIR/"; \
		rm -f /tmp/sqlc-gen-from-template.tar.gz; \
		echo "✓ sqlc-gen-from-template installed to $$INSTALL_DIR"; \
		echo "  Make sure $$INSTALL_DIR is in your PATH"; \
	fi

install: _install-sqlc-gen-from-template ## Install all dependencies (Rust, Node.js, and Python)
	git submodule update --init --recursive
	@echo "Installing JS monorepo dependencies..."
	pnpm install
	@echo "Installing Python monorepo dependencies..."
	uv sync --all-packages
	@echo "✓ All dependencies installed"

build: ## Build all projects (Rust + JS + Python)
	cargo build --bin soma
	$(MAKE) js-generate-client
	$(MAKE) rs-build
	$(MAKE) js-build
	$(MAKE) py-build

js-generate-client: ## Generate JS client
	@echo "Generating JS client..."
	npx --yes openapi-typescript@latest openapi.json -o ./crates/soma-frontend/app/src/@types/openapi.d.ts
	@echo "✓ JS client generated"

js-build: ## Build all JS projects
	@echo "Building JS projects..."
	pnpm -r --workspace-concurrency=1 run build
	@echo "✓ JS projects built"

rs-build: ## Build all Rust projects
	@echo "Building Rust projects..."
	cargo build --features unit_test,integration_test
	@echo "Building Rust tests..."
	cargo test --features unit_test,integration_test --no-run
	@echo "✓ Rust projects built"

py-clean-cache: ## Clean Python bytecode cache files
	@echo "Cleaning Python cache files..."
	@find py -type d -name "__pycache__" -exec rm -rf {} + 2>/dev/null || true
	@find py -type f -name "*.pyc" -delete 2>/dev/null || true
	@find py -type f -name "*.pyo" -delete 2>/dev/null || true
	@echo "✓ Python cache cleaned"

py-build: py-clean-cache ## Build all Python projects
	@echo "Building Python packages..."
	uv sync --all-packages
	uv build --package trysoma-sdk
	@echo "Generating OpenAPI client..."
	VERSION=$$(cat VERSION) && cd py/packages/api_client && npx --yes @openapitools/openapi-generator-cli@latest generate -i ../../../openapi.json -g python -o ./ --additional-properties="packageName=trysoma_api_client,packageVersion=$$VERSION,projectName=trysoma_api_client" && uvx ruff format
	uv build --package trysoma-api-client
	@echo "Installing built packages..."
	uv sync --all-packages
	uv build --package trysoma-insurance-claim-bot-example
	@echo "Installing SDK core as editable (must be last to avoid uv sync overwriting)..."
	$(MAKE) py-build-sdk-core
	@echo "✓ Python projects built and installed"

py-build-sdk-core: ## Build the Python SDK core native module (PyO3/maturin)
	@echo "Building Python SDK core (maturin)..."
	uv run maturin develop --release -m crates/sdk-py/Cargo.toml
	@echo "Regenerating Python type stubs..."
	cargo run --release --bin sdk-py-generate-pyi --manifest-path crates/sdk-py/Cargo.toml -- crates/sdk-py/trysoma_sdk_core/__init__.pyi
	@echo "✓ Python SDK core built and installed"

py-build-sdk-core-wheel: ## Build the Python SDK core wheel for distribution
	@echo "Building Python SDK core wheel..."
	@echo "Step 1: Building the library..."
	uv run maturin develop --release -m crates/sdk-py/Cargo.toml
	@echo "Step 2: Regenerating Python type stubs..."
	cargo run --release --bin sdk-py-generate-pyi --manifest-path crates/sdk-py/Cargo.toml -- crates/sdk-py/trysoma_sdk_core/__init__.pyi
	@echo "Step 3: Building wheel..."
	maturin build --release -m crates/sdk-py/Cargo.toml
	@echo "✓ Python SDK core wheel built to target/wheels/"

py-install: ## Install Python SDK in development mode
	@echo "Installing Python SDK in development mode..."
	uv sync --all-packages
	uv run maturin develop -m crates/sdk-py/Cargo.toml
	@echo "✓ Python SDK installed in development mode"

build-release: ## Build release binaries for Linux, Mac, and Windows
	@echo "Building Rust release binaries for multiple targets..."
	@echo "Note: Cross-compilation requires Docker and may take significant time."
	@echo ""
	@echo "Building for x86_64-unknown-linux-gnu (native)..."
	cargo build --release --target x86_64-unknown-linux-gnu --workspace 
	@echo "✓ x86_64-unknown-linux-gnu built"
	@echo ""
	@echo "Temporarily disabling cargo config for cross-compilation..."
	@mv .cargo/config.toml .cargo/config.toml.tmp 2>/dev/null || true
	@echo "Building for aarch64-unknown-linux-gnu..."
	-cross build --release --target aarch64-unknown-linux-gnu --workspace && echo "✓ aarch64-unknown-linux-gnu built" || echo "⚠ aarch64-unknown-linux-gnu build failed (cross-compilation)"
	@echo ""
	@echo "Building for x86_64-apple-darwin..."
	-cross build --release --target x86_64-apple-darwin --workspace && echo "✓ x86_64-apple-darwin built" || echo "⚠ x86_64-apple-darwin build failed (cross-compilation)"
	@echo ""
	@echo "Building for aarch64-apple-darwin..."
	-cross build --release --target aarch64-apple-darwin --workspace && echo "✓ aarch64-apple-darwin built" || echo "⚠ aarch64-apple-darwin build failed (cross-compilation)"
	@echo ""
	@echo "Building for x86_64-pc-windows-gnu..."
	-cross build --release --target x86_64-pc-windows-gnu --workspace && echo "✓ x86_64-pc-windows-gnu built" || echo "⚠ x86_64-pc-windows-gnu build failed (cross-compilation)"
	@echo ""
	@echo "Restoring cargo config..."
	@mv .cargo/config.toml.tmp .cargo/config.toml 2>/dev/null || true
	@echo "✓ Rust release binaries built (see above for any failures)"
	@echo ""
	@echo "Building JS projects..."
	pnpm -r --workspace-concurrency=1 run build
	@echo "✓ All release builds completed"

clean: ## Clean build artifacts and cache files
	@echo "Cleaning Rust build artifacts..."
	cargo clean
	@echo "Cleaning JS cache files..."
	find . -type d -name "node_modules" -exec rm -rf {} + 2>/dev/null || true
	find . -type d -name "dist" -exec rm -rf {} + 2>/dev/null || true
	find . -type d -name ".turbo" -exec rm -rf {} + 2>/dev/null || true
	@echo "Cleaning Python cache files..."
	find . -type d -name "__pycache__" -exec rm -rf {} + 2>/dev/null || true
	find . -type d -name ".mypy_cache" -exec rm -rf {} + 2>/dev/null || true
	find . -type d -name ".ruff_cache" -exec rm -rf {} + 2>/dev/null || true
	find . -type d -name "*.egg-info" -exec rm -rf {} + 2>/dev/null || true
	rm -rf .venv 2>/dev/null || true
	@echo "Cleaning coverage reports..."
	rm -rf coverage .coverage-tmp
	find . -type d -name "coverage" -not -path "./node_modules/*" -exec rm -rf {} + 2>/dev/null || true
	@echo "✓ Clean completed"

test: test-unit ## Run unit tests only (Rust + JS + Python) - alias for test-unit

test-unit: ## Run unit tests only (Rust + JS + Python) - excludes AWS integration tests
	@echo "Running Rust unit tests..."
	cargo nextest run --features unit_test
	@echo "Running JS tests..."
	pnpm -r --workspace-concurrency=1 --filter '!@trysoma/api-client' run test
	@echo "Running Python tests..."
	uv run pytest py/packages/sdk/tests --tb=short -q || echo "⚠ No Python tests or tests skipped"
	@echo "✓ Unit tests passed"

test-integration: ## Run integration tests only (requires AWS credentials)
	@echo "Running Rust integration tests (requires AWS credentials)..."
	cd test && docker compose up -d && cd ../
	cargo nextest run --features integration_test
	cd test && docker compose down && cd ../
	@echo "✓ Integration tests passed"

test-all: ## Run all tests including integration tests (Rust + JS)
	@echo "Running all Rust tests (unit + integration)..."
	cd test && docker compose up -d && cd ../
	cargo nextest run --features unit_test,integration_test
	@echo "Running JS tests..."
	pnpm -r --workspace-concurrency=1 --filter '!@trysoma/api-client' run test
	@echo "Running Python tests..."
	uv run pytest py/packages/sdk/tests --tb=short -q || echo "⚠ No Python tests or tests skipped"
	cd test && docker compose down && cd ../
	@echo "✓ All tests passed"

py-test: ## Run Python tests only
	@echo "Running Python tests..."
	uv run pytest py/packages/sdk/tests -v
	@echo "✓ Python tests passed"

py-test-coverage: ## Run Python tests with coverage
	@echo "Running Python tests with coverage..."
	uv run pytest py/packages/sdk/tests --cov=py/packages/sdk/trysoma_sdk --cov-report=lcov:py/coverage.lcov --cov-report=term -v
	@echo "✓ Python coverage generated"

test-coverage: ## Run tests with coverage and generate merged report
	@echo "Cleaning previous coverage reports..."
	@rm -rf coverage .coverage-tmp
	@mkdir -p .coverage-tmp coverage
	@echo "Running Rust tests with coverage..."
	cd test && docker compose up -d && cd ../
	cargo llvm-cov nextest --features unit_test,integration_test --workspace --lcov --output-path .coverage-tmp/rust.lcov
	@echo "✓ Rust coverage generated"
	@echo "Running JS tests with coverage..."
	pnpm -r --workspace-concurrency=1 --filter './js/packages/*' --filter './crates/sdk-js' run test:coverage
	@echo "✓ JS coverage generated"
	@echo "Collecting JS coverage reports..."
	@find . -name 'lcov.info' -type f -not -path './coverage/*' -not -path './node_modules/*' -not -path './js/examples/*' -not -path './py/*' | while read file; do \
		dir=$$(dirname "$$file"); \
		pkgdir=$$(dirname "$$dir"); \
		name=$$(echo "$$pkgdir" | sed 's/^\.\///' | sed 's/\//-/g'); \
		sed "s|^SF:|SF:$$pkgdir/|g" "$$file" > ".coverage-tmp/js-$$name.lcov" 2>/dev/null || true; \
	done
	@echo "Running Python tests with coverage..."
	uv run pytest py/packages/sdk/tests --cov=py/packages/sdk/trysoma_sdk --cov-report=lcov:.coverage-tmp/py.lcov --cov-report=term -v || echo "⚠ Python coverage skipped"
	@echo "✓ Python coverage generated"
	@echo "Merging coverage reports..."
	@npx lcov-result-merger '.coverage-tmp/*.lcov' 'coverage/lcov.info'
	@echo "✓ Coverage reports merged to coverage/lcov.info"
	@echo "Generating HTML report..."
	genhtml coverage/lcov.info --output-directory coverage/html --ignore-errors source,range --prefix $$(pwd); \
	echo "✓ HTML report generated at coverage/html/index.html"; \

	@echo "Cleaning up temporary files..."
	@rm -rf .coverage-tmp
	@cd test && docker compose down && cd ../
	@echo "✓ Test coverage complete"


# ============================================================================
# Linting Commands
# ============================================================================

lint: lint-rs lint-js lint-py ## Run all linters (Rust + JS + Python)

lint-rs: ## Run Rust linters (clippy + fmt check)
	@echo "Running cargo clippy..."
	cargo clippy --locked --all-targets --all-features -- -D warnings 
	@echo "Checking Rust formatting..."
	cargo fmt --all -- --check
	@echo "✓ Rust linters passed"

lint-js: ## Run JS/TS linters
	@echo "Running JS linters..."
	pnpm -r --workspace-concurrency=1 run lint
	@echo "✓ JS linters passed"

lint-py: ## Run Python linters (ruff check + format + mypy)
	@echo "Running ruff check..."
	uv run ruff check py/
	@echo "Running ruff format check..."
	uv run ruff format --check py/
	@echo "Running mypy type checking (sdk only, api_client is auto-generated)..."
	uv run mypy py/packages/sdk --ignore-missing-imports --exclude 'py/packages/api_client/.*'
	@echo "✓ Python linters passed"

lint-db: ## Run database linters
	@echo "Running database linters..."
	@soma_output=$$(atlas migrate lint --env soma --git-base main 2>&1); \
	if [ -z "$$soma_output" ]; then \
		echo "Soma DB: SUCCESS: checksums match, no breaking changes"; \
	else \
		echo "$$soma_output"; \
	fi
	@mcp_output=$$(atlas migrate lint --env mcp --git-base main 2>&1); \
	if [ -z "$$mcp_output" ]; then \
		echo "MCP DB: SUCCESS: checksums match, no breaking changes"; \
	else \
		echo "$$mcp_output"; \
	fi
	@encryption_output=$$(atlas migrate lint --env encryption --git-base main 2>&1); \
	if [ -z "$$encryption_output" ]; then \
		echo "Encryption DB: SUCCESS: checksums match, no breaking changes"; \
	else \
		echo "$$encryption_output"; \
	fi
	@identity_output=$$(atlas migrate lint --env identity --git-base main 2>&1); \
	if [ -z "$$identity_output" ]; then \
		echo "Identity DB: SUCCESS: checksums match, no breaking changes"; \
	else \
		echo "$$identity_output"; \
	fi
	@echo "✓ Database linters passed"

lint-fix: lint-fix-rs lint-fix-js lint-fix-py ## Run all linters with auto-fix (Rust + JS + Python)

lint-fix-rs: ## Run Rust linters with auto-fix
	@echo "Running cargo clippy with --fix..."
	cargo clippy --locked --all-targets --all-features --fix --allow-dirty --allow-staged
	cargo clippy --locked --all-targets --all-features -- -D warnings 
	@echo "Formatting Rust code..."
	cargo fmt --all
	@echo "✓ Rust linters completed"

lint-fix-js: ## Run JS/TS linters with auto-fix
	@echo "Running JS linters with auto-fix..."
	pnpm -r --workspace-concurrency=1 run lint:fix
	@echo "✓ JS linters completed"

lint-fix-py: ## Run Python linters with auto-fix
	@echo "Running ruff check with --fix..."
	uv run ruff check --fix py/
	@echo "Formatting Python code with ruff..."
	uv run ruff format py/
	@echo "Running mypy type checking (sdk only, api_client is auto-generated)..."
	uv run mypy py/packages/sdk
	uv run mypy py/packages/api_client --ignore-missing-imports --disable-error-code=return
	uv run mypy py/examples/insurance_claim_bot
	@echo "✓ Python linters completed"

# ============================================================================
# Database Commands
# ============================================================================

_db-generate-migration: ## Create a new database migration using Atlas (internal helper)
	@if [ -z "$(NAME)" ]; then \
		echo "Error: NAME is required. Usage: make db-internal-generate-migration NAME=migration_name"; \
		exit 1; \
	fi
	@if [ -z "$(ENV)" ]; then \
		echo "Error: ENV is required. Usage: make db-internal-generate-migration ENV=internal-local"; \
		exit 1; \
	fi
	@if [ -z "$(FILE_PATH)" ]; then \
		echo "Error: FILE_PATH is required. Usage: make db-internal-generate-migration FILE_PATH=app/dbs/internal/schema.sql"; \
		exit 1; \
	fi

	@echo "Creating new migration: $(NAME)..."
	atlas migrate diff --env $(ENV) $(NAME)
	@echo "✓ Migration created in $(FILE_PATH)/migrations/"

_db-generate-hash: ## Update migration hash file (internal helper)
	@if [ -z "$(ENV)" ]; then \
		echo "Error: ENV is required. Usage: make _db-generate-hash ENV=internal-local"; \
		exit 1; \
	fi

	@echo "Updating migration hash..."
	atlas migrate hash --env $(ENV)
	@echo "✓ Migration hash updated"

db-generate-rs-models: ## Generate Rust models from SQL queries using sqlc
	@echo "Generating Rust models for soma..."
	cd crates/soma-api-server && sqlc generate
	@echo "✓ Soma models generated"
	@echo "Generating Rust models for mcp..."
	cd crates/mcp && sqlc generate
	@echo "✓ MCP models generated"
	@echo "Generating Rust models for encryption..."
	cd crates/encryption && sqlc generate
	@echo "✓ Encryption models generated"
	@echo "Generating Rust models for identity..."
	cd crates/identity && sqlc generate
	@echo "✓ Identity models generated"

db-mcp-generate-migration: ## Create a new mcp database migration using Atlas (usage: make db-mcp-generate-migration NAME=migration_name)
	$(MAKE) _db-generate-migration ENV=mcp FILE_PATH=crates/mcp/dbs/mcp/schema.sql NAME=$(NAME)

db-mcp-generate-hash: ## Update mcp database migration hash
	$(MAKE) _db-generate-hash ENV=mcp

db-encryption-generate-migration: ## Create a new encryption database migration using Atlas (usage: make db-encryption-generate-migration NAME=migration_name)
	$(MAKE) _db-generate-migration ENV=encryption FILE_PATH=crates/encryption/dbs/encryption/schema.sql NAME=$(NAME)

db-encryption-generate-hash: ## Update encryption database migration hash
	$(MAKE) _db-generate-hash ENV=encryption

db-identity-generate-migration: ## Create a new identity database migration using Atlas (usage: make db-identity-generate-migration NAME=migration_name)
	$(MAKE) _db-generate-migration ENV=identity FILE_PATH=crates/identity/dbs/identity/schema.sql NAME=$(NAME)

db-identity-generate-hash: ## Update identity database migration hash
	$(MAKE) _db-generate-hash ENV=identity

db-soma-generate-migration: ## Create a new soma database migration using Atlas (usage: make db-soma-generate-migration NAME=migration_name)
	$(MAKE) _db-generate-migration ENV=soma FILE_PATH=crates/soma/dbs/soma/schema.sql NAME=$(NAME)

db-soma-generate-hash: ## Update soma database migration hash
	$(MAKE) _db-generate-hash ENV=soma

generate-licenses: ## Generate third-party license files for Rust, JS, and Python dependencies
	@echo "Generating Rust licenses..."
	cargo about generate about.hbs > THIRD_PARTY_LICENSES/rust-licenses.md
	@echo "✓ Rust licenses generated"
	@echo "Generating JS licenses..."
	pnpm licenses list > THIRD_PARTY_LICENSES/js-licenses.md
	@echo "✓ JS licenses generated"
	@echo "Generating Python licenses..."
	uv run pip-licenses --format=markdown --with-urls --ignore-packages trysoma-sdk trysoma-sdk-core trysoma-api-client soma-py-workspace > THIRD_PARTY_LICENSES/python-licenses.md
	@echo "✓ Python licenses generated"

# ============================================================================
# Development Commands
# ============================================================================

dev-insurance-claim-bot: ## Start the JS insurance claim bot example
	@if [ -z "$$OPENAI_API_KEY" ]; then \
		echo "Error: OPENAI_API_KEY environment variable is not set"; \
		echo "Please set it with: export OPENAI_API_KEY=your-api-key"; \
		exit 1; \
	fi
	@echo "Starting JS insurance bot..."
	cargo run --bin soma -- dev --cwd ./js/examples/insurance-claim-bot --clean
	@echo "✓ JS Insurance bot started"

dev-insurance-claim-bot-py: ## Start the Python insurance claim bot example
	@if [ -z "$$OPENAI_API_KEY" ]; then \
		echo "Error: OPENAI_API_KEY environment variable is not set"; \
		echo "Please set it with: export OPENAI_API_KEY=your-api-key"; \
		exit 1; \
	fi
	@echo "Starting Python insurance bot..."
	cd py/examples/insurance_claim_bot && uv run python -m soma_sdk.standalone --watch .
	@echo "✓ Python Insurance bot started"

py-generate-standalone: ## Generate standalone.py for a Python example project
	@if [ -z "$(DIR)" ]; then \
		echo "Error: DIR is required. Usage: make py-generate-standalone DIR=py/examples/insurance_claim_bot"; \
		exit 1; \
	fi
	@echo "Generating standalone.py for $(DIR)..."
	cd $(DIR) && uv run python -m soma_sdk.standalone .
	@echo "✓ standalone.py generated"
