#![no_std]

extern crate common;
use common::ext::*;
use common::value::Value;

#[no_mangle]
pub fn call() {
    let x = Value::Int32(1234);
    let y = Value::Int32(8910);
    let x_key = new_uref();
    let y_key = new_uref();
    write(&x_key, &x);
    write(&y_key, &y);
    let xr = read(&x_key);
    let yr = read(&y_key);
    write(&new_uref(), &xr);
    write(&new_uref(), &yr);
    assert_eq!(x, xr);
    assert_eq!(y, yr);
}
