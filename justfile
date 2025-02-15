build:
	cargo build

fmt:
	cargo fmt --all

install-dev-deps:
	cargo install cargo-check
	cargo install cargo-watch

watch:
	cargo watch --exec 'run' | bunyan
