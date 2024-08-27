build:
	cargo build
build_daemon:
	cargo build --bin daemon --features daemon
	cargo build --bin routing --features routing
build_release:
	cargo build --release --bin daemon --features daemon
	cargo build --release --bin routing --features routing
aarch64:
	if [ -d "bin" ] ; then rm -rf bin/; fi
	mkdir bin
	cross build --release --target aarch64-unknown-linux-gnu --bin daemon --features daemon
	cp target/aarch64-unknown-linux-gnu/release/daemon bin/
	cross build --release --target aarch64-unknown-linux-gnu --bin routing --features routing
	cp target/aarch64-unknown-linux-gnu/release/routing bin/
