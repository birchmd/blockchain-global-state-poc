#![no_std]
#![feature(alloc)]

#[macro_use]
extern crate alloc;
use alloc::string::String;

extern crate common;
use common::bytesrepr::BytesRepr;
use common::ext::*;
use common::value::Value;

fn plus_one(x: i32) -> i32 {
    x + 1
}

#[no_mangle]
pub extern "C" fn plus_one_ext() {
    let x: i32 = get_arg(0);
    let y = plus_one(x);
    ret(&y);
}

#[no_mangle]
pub extern "C" fn call() {
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

    let export_name = String::from("plus_one_ext");
    let hash = store_function(&export_name);
    let contract = read(&hash);
	//TODO: add some convenient macro to common to do this conversion
    let args = vec![3i32.to_bytes()];
    let result: i32 = call_contract(&contract, &args);
    assert_eq!(result, 4);
}
