LINT_PARAMS := $(shell cat .lints | cut -f1 -d"#" | tr '\n' ' ')

build:
	cargo build --bin daemon --features daemon
	cargo build --bin routing --features routing
	cargo build --bin runner --features runner
build-release:
	cargo build --release --bin daemon --features daemon
	cargo build --release --bin routing --features routing
	cargo build --release --bin runner --features runner

aarch64:
	cross build --release --target aarch64-unknown-linux-gnu --bin daemon --features daemon
	cross build --release --target aarch64-unknown-linux-gnu --bin routing --features routing
	cross build --release --target aarch64-unknown-linux-gnu --bin runner --features runner

install: clean-bin build
	mkdir bin
	cp target/debug/daemon bin/
	cp target/debug/routing bin/
	cp target/debug/runner bin/
install-aarch64: clean-bin aarch64
	mkdir bin
	cp target/aarch64-unknown-linux-gnu/release/daemon bin/
	cp target/aarch64-unknown-linux-gnu/release/routing bin/
	cp target/aarch64-unknown-linux-gnu/release/runner bin/

clean: clean-target clean-bin
clean-target:
	rm -rf target
clean-bin:
	rm -rf bin

clippy:
	cargo clippy --all-targets -- -D warnings $(LINT_PARAMS)
