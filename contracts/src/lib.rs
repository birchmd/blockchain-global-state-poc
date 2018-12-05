#![no_std]
#![feature(alloc)]

#[macro_use]
extern crate alloc;

extern crate common;
use common::ext::*;
use common::value::Value;

#[no_mangle]
pub extern fn call() {
    let x = 1234;
    let y = 8910;
    let x_key = new_uref();
    write(&x_key, &Value::ListInt32(vec![x, y]));
    let xy = read(&x_key);
    assert_eq!(xy, Value::ListInt32(vec![x, y]));
}
