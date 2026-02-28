.PHONY: lint generate breaking all clean \
       rust-build rust-check rust-fmt rust-clippy rust-test rust-audit \
       python-install python-check python-fmt python-lint python-audit \
       core-build core-test core-fmt core-clippy core-doc \
       docker-build docker-up docker-down \
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
	cd python && uv sync

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

# ─── Aggregate ────────────────────────────────────────────────────────
typos:
	typos

check: lint core-fmt core-clippy core-build core-test python-check typos

all: lint generate core-build python-install

clean:
	rm -rf rust/src/gen python/src/gen rust/target python/.venv target/

pre-commit:
	pre-commit install
	pre-commit install --hook-type commit-msg
