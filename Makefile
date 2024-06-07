build:
	cargo build
build_daemon:
	cargo build --bin daemon --features daemon
build_release:
	cargo build --release --bin daemon --features daemon
aarch64:
	mkdir bin
	cross build --release --target aarch64-unknown-linux-gnu --bin daemon --features daemon
	cp target/aarch64-unknown-linux-gnu/release/daemon bin/
	rm -rf bin