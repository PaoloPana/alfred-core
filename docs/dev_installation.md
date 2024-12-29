# Dev installation

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
