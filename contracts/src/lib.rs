#![no_std]

extern crate common;
use common::ext::*;
use common::value::Value;

#[no_mangle]
pub fn call() {
    let x = Value::Int32(0);
    let key = unsafe{ new_uref() };
    unsafe { write(&key, &x) };
    let y = unsafe { read(&key) };
    unsafe { write(&new_uref(), &y) };
    assert_eq!(x, y);
}
