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
    let list = Value::ListInt32(vec![x, y]);
    write(&x_key, &list);
    let list_read = read(&x_key);
    assert_eq!(list, list_read);

    let hw = alloc::string::String::from("Hello, World!");
    let s = Value::String(hw);
    write(&x_key, &s);
    let s_read = read(&x_key);
    assert_eq!(s, s_read);
}
