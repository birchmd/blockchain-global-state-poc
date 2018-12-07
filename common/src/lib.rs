#![no_std]
#![feature(alloc, allocator_api, core_intrinsics, lang_items, alloc_error_handler)]

extern crate alloc;
extern crate wee_alloc;

#[cfg(not(feature = "std"))]
#[global_allocator]
pub static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub mod bytesrepr;
pub mod key;
pub mod value;

mod ext_ffi {
    extern "C" {
        //TODO: replace size_of* idiom with one in which the calls are explicitly
        //      paired. E.g. read_value(key_ptr: *const u8, key_size: usize) -> usize;
        //                   get_read(value_ptr: *mut u8, value_size: usize);
        //      Note that the second call does not make sense except after the first one.
        pub fn size_of_value(key_ptr: *const u8, key_size: usize) -> usize;
        pub fn read(key_ptr: *const u8, key_size: usize, value_ptr: *mut u8, value_size: usize);
        pub fn write(key_ptr: *const u8, key_size: usize, value_ptr: *const u8, value_size: usize);
        pub fn add(key_ptr: *const u8, key_size: usize, value_ptr: *const u8, value_size: usize);
        pub fn new_uref(key_ptr: *mut u8);
        pub fn function_size(name_ptr: *const u8, name_size: usize) -> usize;
        pub fn function_bytes(
            name_ptr: *const u8,
            name_size: usize,
            dest_ptr: *mut u8,
            dest_size: usize,
        );
        pub fn size_of_arg(i: u32) -> usize;
        pub fn get_arg(i: u32, dest: *mut u8, size: usize);
        pub fn ret(value_ptr: *const u8, value_size: usize) -> !;
        pub fn size_of_call_result(
            fn_ptr: *const u8,
            fn_size: usize,
            args_ptr: *const u8,
            args_size: usize,
        ) -> usize;
        pub fn call_contract(
            fn_ptr: *const u8,
            fn_size: usize,
            args_ptr: *const u8,
            args_size: usize,
            res_ptr: *mut u8,
            res_size: usize,
        );
    }
}

pub mod ext {
    extern crate blake2;

    use super::alloc::alloc::{Alloc, Global};
    use super::alloc::string::String;
    use super::alloc::vec::Vec;
    use super::ext_ffi;
    use blake2::digest::{Input, VariableOutput};
    use blake2::VarBlake2b;
    use crate::bytesrepr::{deserialize, BytesRepr};
    use crate::key::{Key, UREF_SIZE};
    use crate::value::Value;

    fn alloc_bytes(n: usize) -> *mut u8 {
        Global.alloc_array(n).unwrap().as_ptr()
    }

    fn to_ptr<T: BytesRepr>(t: &T) -> (*const u8, usize, Vec<u8>) {
        let bytes = t.to_bytes();
        let ptr = bytes.as_ptr();
        let size = bytes.len();
        (ptr, size, bytes)
    }

    pub fn read(key: &Key) -> Value {
        //Note: _bytes is necessary to keep the Vec<u8> in scope. If _bytes is
        //      dropped then key_ptr becomes invalid.
        let (key_ptr, key_size, _bytes) = to_ptr(key);
        let value_size = unsafe { ext_ffi::size_of_value(key_ptr, key_size) };
        let value_ptr = alloc_bytes(value_size);
        let value_bytes = unsafe {
            ext_ffi::read(key_ptr, key_size, value_ptr, value_size);
            core::slice::from_raw_parts(value_ptr, value_size)
        };
        deserialize(value_bytes).unwrap()
    }

    pub fn write(key: &Key, value: &Value) {
        let (key_ptr, key_size, _bytes) = to_ptr(key);
        let (value_ptr, value_size, _bytes2) = to_ptr(value);
        unsafe {
            ext_ffi::write(key_ptr, key_size, value_ptr, value_size);
        }
    }

    pub fn add(key: &Key, value: &Value) {
        let (key_ptr, key_size, _bytes) = to_ptr(key);
        let (value_ptr, value_size, _bytes2) = to_ptr(value);
        unsafe {
            ext_ffi::add(key_ptr, key_size, value_ptr, value_size);
        }
    }

    pub fn new_uref() -> Key {
        let key_ptr = alloc_bytes(UREF_SIZE);
        let slice = unsafe {
            ext_ffi::new_uref(key_ptr);
            core::slice::from_raw_parts(key_ptr, UREF_SIZE)
        };
        deserialize(slice).unwrap()
    }

    fn fn_bytes_by_name(name: &String) -> Vec<u8> {
        let (name_ptr, name_size, _bytes) = to_ptr(name);
        let fn_size = unsafe { ext_ffi::function_size(name_ptr, name_size) };
        let fn_ptr = alloc_bytes(fn_size);
        unsafe {
            ext_ffi::function_bytes(name_ptr, name_size, fn_ptr, fn_size);
            Vec::from_raw_parts(fn_ptr, fn_size, fn_size)
        }
    }

    pub fn fn_by_name(name: &String) -> Value {
        let fn_bytes = fn_bytes_by_name(name);
        Value::Contract(fn_bytes)
    }

    pub fn store_function(name: &String) -> Key {
        let fn_bytes = fn_bytes_by_name(name);
        let mut hasher = VarBlake2b::new(32).unwrap();
        hasher.input(&fn_bytes);
        let mut fn_hash = [0u8; 32];
        hasher.variable_result(|hash| fn_hash.clone_from_slice(hash));
        let key = Key::Hash(fn_hash);
        let value = Value::Contract(fn_bytes);
        write(&key, &value);
        key
    }

    pub fn get_arg<T: BytesRepr>(i: u32) -> T {
        let arg_size = unsafe { ext_ffi::size_of_arg(i) };
        let dest_ptr = alloc_bytes(arg_size);
        let arg_bytes = unsafe {
            ext_ffi::get_arg(i, dest_ptr, arg_size);
            core::slice::from_raw_parts(dest_ptr, arg_size)
        };
        deserialize(arg_bytes).unwrap()
    }

    pub fn ret<T: BytesRepr>(t: &T) -> ! {
        let (ptr, size, _bytes) = to_ptr(t);
        unsafe {
            ext_ffi::ret(ptr, size);
        }
    }

    pub fn call_contract<T: BytesRepr>(contract: &Value, args: &Vec<Vec<u8>>) -> T {
        if let Value::Contract(c) = contract {
            let (fn_ptr, fn_size, _bytes) = to_ptr(c);
            let (args_ptr, args_size, _bytes2) = to_ptr(args);
            let res_size =
                unsafe { ext_ffi::size_of_call_result(fn_ptr, fn_size, args_ptr, args_size) };
            let res_ptr = alloc_bytes(res_size);
            let res_bytes = unsafe {
                ext_ffi::call_contract(fn_ptr, fn_size, args_ptr, args_size, res_ptr, res_size);
                core::slice::from_raw_parts(res_ptr, res_size)
            };
            deserialize(res_bytes).unwrap()
        } else {
            panic!("{:?} is not a contract!", contract);
        }
    }
}

#[cfg(not(feature = "std"))]
pub mod handlers {

    #[panic_handler]
    #[no_mangle]
    pub fn panic(_info: &::core::panic::PanicInfo) -> ! {
        unsafe {
            ::core::intrinsics::abort();
        }
    }

    #[alloc_error_handler]
    #[no_mangle]
    pub extern "C" fn oom(_: ::core::alloc::Layout) -> ! {
        unsafe {
            ::core::intrinsics::abort();
        }
    }

    #[lang = "eh_personality"]
    extern "C" fn eh_personality() {}
}
