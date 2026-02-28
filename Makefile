.PHONY: lint generate breaking all clean \
       rust-build rust-check rust-fmt rust-clippy rust-test rust-audit \
       python-install python-check python-fmt python-lint python-audit \
       check pre-commit typos

# ─── Proto ────────────────────────────────────────────────────────────
lint:
	buf lint

generate:
	buf generate

breaking:
	buf breaking --against '.git#branch=main'

# ─── Rust ─────────────────────────────────────────────────────────────
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

# ─── Aggregate ────────────────────────────────────────────────────────
typos:
	typos

check: lint rust-fmt rust-clippy rust-build rust-test python-check typos

all: lint generate rust-build python-install

clean:
	rm -rf rust/src/gen python/src/gen rust/target python/.venv

pre-commit:
	pre-commit install
	pre-commit install --hook-type commit-msg
