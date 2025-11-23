.PHONY: help install clean build build-release test test-coverage \
	lint lint-js lint-rs lint-fix lint-fix-js lint-fix-rs \
	db-internal-generate-migration db-internal-generate-hash db-generate-rs-models \
	db-bridge-generate-migration db-bridge-generate-hash db-soma-generate-migration db-soma-generate-hash \
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

install: _install-sqlc-gen-from-template ## Install all dependencies (Rust and Node.js)
	git submodule update --init --recursive
	@echo "Installing JS monorepo dependencies..."
	pnpm install
	@echo "✓ All dependencies installed"

build: ## Build all projects (Rust + JS)
	$(MAKE) js-build
	$(MAKE) rs-build

js-build: ## Build all JS projects
	@echo "Building JS projects..."
	pnpm -r --workspace-concurrency=1 run build
	@echo "✓ JS projects built"

rs-build: ## Build all Rust projects
	@echo "Building Rust projects..."
	cargo build
	@echo "✓ Rust projects built"

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
	@echo "Cleaning coverage reports..."
	rm -rf coverage .coverage-tmp
	find . -type d -name "coverage" -not -path "./node_modules/*" -exec rm -rf {} + 2>/dev/null || true
	@echo "✓ Clean completed"

test: ## Run all tests (Rust + JS)
	@echo "Running Rust tests..."
	cargo nextest run
	@echo "Running JS tests..."
	pnpm -r --workspace-concurrency=1 run test
	@echo "✓ All tests passed"

test-coverage: ## Run tests with coverage and generate merged report
	@echo "Cleaning previous coverage reports..."
	@rm -rf coverage .coverage-tmp
	@mkdir -p .coverage-tmp coverage
	@echo "Running Rust tests with coverage..."
	cargo llvm-cov nextest --all-features --workspace  --lcov --output-path .coverage-tmp/rust.lcov
	@echo "✓ Rust coverage generated"
	@echo "Running JS tests with coverage..."
	pnpm -r --workspace-concurrency=1 --filter './js/packages/*' --filter './crates/sdk-js' run test:coverage
	@echo "✓ JS coverage generated"
	@echo "Collecting JS coverage reports..."
	@find . -name 'lcov.info' -type f -not -path './coverage/*' -not -path './node_modules/*' -not -path './js/examples/*' | while read file; do \
		dir=$$(dirname "$$file"); \
		pkgdir=$$(dirname "$$dir"); \
		name=$$(echo "$$pkgdir" | sed 's/^\.\///' | sed 's/\//-/g'); \
		sed "s|^SF:|SF:$$pkgdir/|g" "$$file" > ".coverage-tmp/js-$$name.lcov" 2>/dev/null || true; \
	done
	@echo "Merging coverage reports..."
	@npx lcov-result-merger '.coverage-tmp/*.lcov' 'coverage/lcov.info'
	@echo "✓ Coverage reports merged to coverage/lcov.info"
	@echo "Generating HTML report..."
	genhtml coverage/lcov.info --output-directory coverage/html --ignore-errors source,range --prefix $$(pwd); \
	echo "✓ HTML report generated at coverage/html/index.html"; \

	@echo "Cleaning up temporary files..."
	@rm -rf .coverage-tmp
	@echo "✓ Test coverage complete"


# ============================================================================
# Linting Commands
# ============================================================================

lint: lint-rs lint-js ## Run all linters (Rust + JS)

lint-rs: ## Run Rust linters (clippy + fmt check)
	@echo "Running cargo clippy..."
	cargo clippy --all-targets --all-features -- -D warnings
	@echo "Checking Rust formatting..."
	cargo fmt --all -- --check
	@echo "✓ Rust linters passed"

lint-js: ## Run JS/TS linters
	@echo "Running JS linters..."
	pnpm -r --workspace-concurrency=1 run lint
	@echo "✓ JS linters passed"

lint-db: ## Run database linters
	@echo "Running database linters..."
	@soma_output=$$(atlas migrate lint --env soma --git-base main 2>&1); \
	if [ -z "$$soma_output" ]; then \
		echo "Soma DB: SUCCESS: checksums match, no breaking changes"; \
	else \
		echo "$$soma_output"; \
	fi
	@bridge_output=$$(atlas migrate lint --env bridge --git-base main 2>&1); \
	if [ -z "$$bridge_output" ]; then \
		echo "Bridge DB: SUCCESS: checksums match, no breaking changes"; \
	else \
		echo "$$bridge_output"; \
	fi
	@echo "✓ Database linters passed"

lint-fix: lint-fix-rs lint-fix-js ## Run all linters with auto-fix (Rust + JS)

lint-fix-rs: ## Run Rust linters with auto-fix
	@echo "Running cargo clippy with --fix..."
	cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged
	@echo "Formatting Rust code..."
	cargo fmt --all
	@echo "✓ Rust linters completed"

lint-fix-js: ## Run JS/TS linters with auto-fix
	@echo "Running JS linters with auto-fix..."
	pnpm -r --workspace-concurrency=1 run lint:fix
	@echo "✓ JS linters completed"

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
	@echo "Generating Rust models..."
	cd crates/soma && sqlc generate
	@echo "Generating Rust models..."
	cd crates/bridge && sqlc generate
	@echo "✓ Rust models generated"

db-bridge-generate-migration: ## Create a new bridge database migration using Atlas (usage: make db-bridge-generate-migration NAME=migration_name)
	$(MAKE) _db-generate-migration ENV=bridge FILE_PATH=crates/bridge/dbs/bridge/schema.sql NAME=$(NAME)

db-bridge-generate-hash: ## Update bridge database migration hash
	$(MAKE) _db-generate-hash ENV=bridge

db-soma-generate-migration: ## Create a new soma database migration using Atlas (usage: make db-soma-generate-migration NAME=migration_name)
	$(MAKE) _db-generate-migration ENV=soma FILE_PATH=crates/soma/dbs/soma/schema.sql NAME=$(NAME)

db-soma-generate-hash: ## Update soma database migration hash
	$(MAKE) _db-generate-hash ENV=soma


# ============================================================================
# Development Commands
# ============================================================================

dev-insurance-claim-bot: ## Start the insurance claim bot
	@if [ -z "$$OPENAI_API_KEY" ]; then \
		echo "Error: OPENAI_API_KEY environment variable is not set"; \
		echo "Please set it with: export OPENAI_API_KEY=your-api-key"; \
		exit 1; \
	fi
	@echo "Starting insurance bot..."
	cargo run --bin soma -- dev --src-dir ./js/examples/insurance-claim-bot --clean
	@echo "✓ Insurance bot started"
