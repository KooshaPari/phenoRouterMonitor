.PHONY: lint generate breaking all clean \
       rust-build rust-check rust-fmt rust-clippy rust-test rust-audit \
       python-install python-check python-fmt python-lint python-audit \
       core-build core-test core-fmt core-clippy core-doc \
       docker-build docker-up docker-down \
       test test-unit test-bdd test-bdd-python test-contracts test-integration test-integration-inner \
       check pre-commit typos

# ─── Proto ────────────────────────────────────────────────────────────
lint:
	buf lint

generate:
	buf generate

breaking:
	buf breaking --against '.git#branch=main'

# ─── Rust (proto crate only) ─────────────────────────────────────────
rust-build:
	cd rust && cargo build

rust-check:
	cd rust && cargo check

rust-fmt:
	cd rust && cargo fmt --check

rust-clippy:
	cd rust && cargo clippy --all-targets -- -D warnings

rust-test:
	cd rust && cargo test

rust-audit:
	cd rust && cargo deny check

rust-machete:
	cd rust && cargo machete

# ─── Core workspace ──────────────────────────────────────────────────
core-build:
	cargo build --workspace

core-test:
	cargo test --workspace

core-fmt:
	cargo fmt --all -- --check

core-clippy:
	cargo clippy --workspace -- -D warnings

core-doc:
	cargo doc --no-deps --workspace

# ─── Python ───────────────────────────────────────────────────────────
python-install:
	cd python && uv sync --all-extras

python-test:
	cd python && uv run pytest tests/ -v

python-fmt:
	cd python && uvx ruff format --check .

python-lint:
	cd python && uvx ruff check .

python-audit:
	cd python && uvx pip-audit

python-check: python-install python-fmt python-lint

# ─── Docker ───────────────────────────────────────────────────────────
docker-build:
	docker-compose build

docker-up:
	docker-compose up -d

docker-down:
	docker-compose down

# ─── gRPC server targets ──────────────────────────────────────────────
grpc-test:
	cargo test -p agileplus-grpc

# ─── BDD / acceptance tests ───────────────────────────────────────────
test-unit:
	cargo test --workspace --exclude agileplus-bdd

test-bdd:
	cargo test --package agileplus-bdd --test bdd

test-bdd-python:
	cd python && uv run behave tests/bdd/ --no-capture

test-contracts:
	cargo test -p agileplus-grpc --test pact_schema
	cd python && uv run pytest tests/contract/ -v

# ─── Full-stack integration tests (requires Docker) ───────────────────
test-integration:
	docker compose -f tests/integration/docker-compose.test.yml up --build --abort-on-container-exit
	docker compose -f tests/integration/docker-compose.test.yml down -v

# Runs INSIDE the test-runner container (invoked by docker compose)
test-integration-inner:
	cargo test --test test_full_workflow -- --include-ignored --test-threads=1
	cd python && uv run pytest tests/integration/ -v

# ─── Master test target ───────────────────────────────────────────────
## Runs all non-Docker tests: unit, BDD (Rust + Python), contracts.
## Add 'test-integration' manually when Docker is available.
test: test-unit test-bdd test-bdd-python test-contracts python-test

# ─── Aggregate ────────────────────────────────────────────────────────
typos:
	typos

check: lint core-fmt core-clippy core-build test python-check typos

all: lint generate core-build python-install

clean:
	rm -rf rust/src/gen python/src/gen rust/target python/.venv target/

pre-commit:
	pre-commit install
	pre-commit install --hook-type commit-msg
