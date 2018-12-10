extern crate common;
extern crate storage;
extern crate wasmi;
extern crate parity_wasm;
extern crate pwasm_utils;

use common::key::Key;
use common::value::{Account, Value};
use storage::transform::Transform;
use storage::GlobalState;
use parity_wasm::elements::{deserialize_buffer, Module};

pub mod engine;

use std::fs::File;

fn load_wasm(path: &str) -> Module {
    use std::io::prelude::*;
    let mut file = File::open(path).unwrap();
    let mut wasm_buf = Vec::new();
    file.read_to_end(&mut wasm_buf).unwrap();
    deserialize_buffer(&wasm_buf).unwrap()
}

fn main() {
    let module = load_wasm("../contracts/target/readwrite.wasm");
    let mut gs = storage::InMemGS::new();
    let account_addr = [0u8; 20];
    let account = Account::new([0u8; 32], 0, Vec::new());
    let transform = Transform::Write(Value::Acct(account));
    gs.apply(Key::Account(account_addr), transform).unwrap();

    let result = engine::exec(module, account_addr, &gs);

    println!("Result is {:?}", result);
}
