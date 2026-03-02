.PHONY: build test proto-gen docker-build fmt clippy clean check

# ─── Build ────────────────────────────────────────────────────────────────────
build:
	cargo build --workspace

# ─── Test ─────────────────────────────────────────────────────────────────────
test:
	cargo test --workspace

# ─── Proto codegen ────────────────────────────────────────────────────────────
# Regenerate Rust bindings from proto/agileplus/v1/agents.proto.
# Bindings are produced by tonic-build at compile time via build.rs;
# this target forces a rebuild of the service crate to re-run build.rs.
proto-gen:
	touch crates/agileplus-agent-service/build.rs
	cargo build -p agileplus-agent-service

# ─── Docker ───────────────────────────────────────────────────────────────────
docker-build:
	docker build -t agileplus-agent-service:latest .

# ─── Linting / formatting ─────────────────────────────────────────────────────
fmt:
	cargo fmt --all -- --check

clippy:
	cargo clippy --all-targets --all-features -- -D warnings

check: fmt clippy test

# ─── Clean ────────────────────────────────────────────────────────────────────
clean:
	cargo clean
