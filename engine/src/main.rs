extern crate wasmi;

pub mod engine;

use std::fs::File;

fn load_wasm(path: &str) -> wasmi::Module {
    use std::io::prelude::*;
    let mut file = File::open(path).unwrap();
    let mut wasm_buf = Vec::new();
    file.read_to_end(&mut wasm_buf).unwrap();
    wasmi::Module::from_buffer(&wasm_buf).unwrap()
}

fn main() {
    let module = load_wasm("/home/birchmd/cpr-blockchain-poc/contracts/readwrite.wasm");
    let gs = storage::InMemGS::new();
    let account: [u8; 20] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    let result = engine::exec(module, account, &gs);

    println!("Result is {:?}", result);
}
