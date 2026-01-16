.PHONY: help install clean build build-release \
	test test-unit test-integration test-all test-coverage \
	lint lint-rs lint-db lint-fix lint-fix-rs \
	db-generate-rs-models \
	db-tool-generate-migration db-tool-generate-hash \
	db-encryption-generate-migration db-encryption-generate-hash \
	db-environment-generate-migration db-environment-generate-hash \
	db-identity-generate-migration db-identity-generate-hash \
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

install: _install-sqlc-gen-from-template ## Install all dependencies (Rust)
	git submodule update --init --recursive
	@echo "✓ All dependencies installed"

build: ## Build all projects (Rust)
	@echo "Building Rust projects..."
	cargo build
	@echo "Building Rust tests..."
	cargo test --no-run
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

clean: ## Clean build artifacts and cache files
	@echo "Cleaning Rust build artifacts..."
	cargo clean
	@echo "Cleaning coverage reports..."
	rm -rf coverage .coverage-tmp
	@echo "✓ Clean completed"

test:
	@echo "Running all Rust tests (unit + integration)..."
	cd test && docker compose up -d && cd ../
	cargo nextest run
	cd test && docker compose down && cd ../
	@echo "✓ All tests passed"

test-coverage: ## Run Rust tests with coverage
	@echo "Cleaning previous coverage reports..."
	@rm -rf coverage
	@mkdir -p coverage
	@echo "Running Rust tests with coverage..."
	cd test && docker compose up -d && cd ../
	cargo llvm-cov nextest --workspace --lcov --output-path coverage/lcov.info
	@echo "✓ Rust coverage generated"
	@echo "Generating HTML report..."
	genhtml coverage/lcov.info --output-directory coverage/html --ignore-errors source,range --prefix $$(pwd); \
	echo "✓ HTML report generated at coverage/html/index.html"; \
	@cd test && docker compose down && cd ../
	@echo "✓ Test coverage complete"


# ============================================================================
# Linting Commands
# ============================================================================

lint: lint-rs ## Run all linters (Rust)

lint-rs: ## Run Rust linters (clippy + fmt check)
	@echo "Running cargo clippy..."
	cargo clippy --locked --all-targets --all-features -- -D warnings
	@echo "Checking Rust formatting..."
	cargo fmt --all -- --check
	@echo "✓ Rust linters passed"

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
	@environment_output=$$(atlas migrate lint --env environment --git-base main 2>&1); \
	if [ -z "$$environment_output" ]; then \
		echo "Environment DB: SUCCESS: checksums match, no breaking changes"; \
	else \
		echo "$$environment_output"; \
	fi
	@echo "✓ Database linters passed"

lint-fix: lint-fix-rs ## Run all linters with auto-fix (Rust)

lint-fix-rs: ## Run Rust linters with auto-fix
	@echo "Running cargo clippy with --fix..."
	cargo clippy --locked --all-targets --all-features --fix --allow-dirty --allow-staged
	cargo clippy --locked --all-targets --all-features -- -D warnings
	@echo "Formatting Rust code..."
	cargo fmt --all
	@echo "✓ Rust linters completed"

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
	@echo "Generating Rust models for environment..."
	cd crates/environment && sqlc generate
	@echo "✓ Environment models generated"

db-tool-generate-migration: ## Create a new tool database migration using Atlas (usage: make db-tool-generate-migration NAME=migration_name)
	$(MAKE) _db-generate-migration ENV=tool FILE_PATH=crates/tool/dbs/tool/schema.sql NAME=$(NAME)

db-tool-generate-hash: ## Update tool database migration hash
	$(MAKE) _db-generate-hash ENV=tool

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

db-environment-generate-migration: ## Create a new environment database migration using Atlas (usage: make db-environment-generate-migration NAME=migration_name)
	$(MAKE) _db-generate-migration ENV=environment FILE_PATH=crates/environment/dbs/environment/schema.sql NAME=$(NAME)

db-environment-generate-hash: ## Update environment database migration hash
	$(MAKE) _db-generate-hash ENV=environment

generate-licenses: ## Generate third-party license files for Rust dependencies
	@echo "Generating Rust licenses..."
	cargo about generate about.hbs > THIRD_PARTY_LICENSES/rust-licenses.md
	@echo "✓ Rust licenses generated"
