fmt:
	cargo fmt --all -- --check

fmt-fix:
	cargo fmt --all

clippy:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

build:
	cargo build --release

test:
	cargo test --workspace --all-features

check: fmt clippy test
