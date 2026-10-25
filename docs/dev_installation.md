# Dev installation

## Tests
`cargo test` runs the unit tests only. The integration tests (`tests/mqtt.rs`) need an MQTT 5 broker and run when
`ALFRED_TEST_BROKER_URL` is set (this is how CI runs them, see `.github/mosquitto-ci.conf`):
```shell
mosquitto -c .github/mosquitto-ci.conf &
ALFRED_TEST_BROKER_URL=mqtt://127.0.0.1:1883 cargo test
```

## Cross compile
### Installation
 - Install docker (https://docs.docker.com/engine/install/ubuntu/)
 - Install docker rootless (https://docs.docker.com/engine/security/rootless/)
```shell
cargo install cross --git https://github.com/cross-rs/cross
```

### Build
```shell
make aarch64
```
