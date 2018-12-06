extern crate common;
extern crate storage;
extern crate wasmi;

use common::key::Key;
use common::value::{Account, Value};
use storage::transform::Transform;
use storage::GlobalState;

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
    let module = load_wasm("../contracts/target/readwrite.wasm");
    let mut gs = storage::InMemGS::new();
    let account_addr = [0u8; 20];
    let account = Account::new([0u8; 32], 0, Vec::new());
    let transform = Transform::Write(Value::Acct(account));
    gs.apply(Key::Account(account_addr), transform).unwrap();

    let result = engine::exec(module, account_addr, &gs);

    println!("Result is {:?}", result);
}
