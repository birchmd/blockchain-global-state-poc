A simple PoC for the global state proposal described on the [Casper Labs wiki](https://casperlabs.atlassian.net/wiki/spaces/~167734600/pages/8388609/A+proposal+for+the+architecture+of+the+Blockchain).

To compile and run:
```shell
cd contracts/
cargo build --target wasm32-unknown-unknown --release
cd ../wasm_prep/
cargo build
./target/debug/wasm_prep ../contracts/target readwrite
cd ../engine/
cargo run
```
