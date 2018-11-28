#![no_std]

extern crate common;
use common::ext::*;
use common::key::Key;
use common::value::Value;

#[no_mangle]
pub fn call() {
    let x = Value::Int32(0);
    let account = Key::Account([0u8; 20]);
    unsafe { write(&account, &x) };
    let y = unsafe { read(&account) };
    assert_eq!(x, y);
}
