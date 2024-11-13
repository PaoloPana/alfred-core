LINT_PARAMS := $(shell cat .lints | cut -f1 -d"#" | tr '\n' ' ')

build:
	cargo build --bin daemon --bin routing --bin runner --bin cron --bin logs --all-features
build-release:
	cargo build --release --bin daemon --bin routing --bin runner --bin cron --bin logs --all-features

aarch64:
	cross build --release --target aarch64-unknown-linux-gnu --bin daemon --bin routing --bin runner --bin cron --bin logs --all-features

install: clean-bin build
	mkdir bin
	cp target/debug/daemon bin/
	cp target/debug/routing bin/
	cp target/debug/runner bin/
	cp target/debug/cron bin/
	cp target/debug/logs bin/
install-aarch64: clean-bin aarch64
	mkdir bin
	cp target/aarch64-unknown-linux-gnu/release/daemon bin/
	cp target/aarch64-unknown-linux-gnu/release/routing bin/
	cp target/aarch64-unknown-linux-gnu/release/runner bin/
	cp target/aarch64-unknown-linux-gnu/release/cron bin/
	cp target/aarch64-unknown-linux-gnu/release/logs bin/

clean: clean-target clean-bin
clean-target:
	rm -rf target
clean-bin:
	rm -rf bin

clippy:
	cargo clippy --all-targets --all-features -- -D warnings $(LINT_PARAMS)

clippy-fix:
	__CARGO_FIX_YOLO=1 cargo clippy --fix --allow-staged --all-targets --all-features -- -D warnings $(LINT_PARAMS)
