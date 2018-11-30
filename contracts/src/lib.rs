#![no_std]

extern crate common;
use common::ext::*;
use common::value::Value;

#[no_mangle]
pub fn call() {
    let x = 1234;
    let y = 8910;
    let x_key = new_uref();
    write(&x_key, &Value::Int32(x));
    add(&x_key, &Value::Int32(y));
    let xy = read(&x_key);
    assert_eq!(xy, Value::Int32(x + y));
}
