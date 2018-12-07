#![no_std]
#![feature(alloc, allocator_api, core_intrinsics, lang_items, alloc_error_handler)]

extern crate alloc;
extern crate wee_alloc;

#[cfg(not(feature = "std"))]
#[global_allocator]
pub static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub mod key;
pub mod bytesrepr;
pub mod value;

mod ext_ffi {
    extern "C" {
        pub fn size_of_value(key_ptr: *const u8, key_size: usize) -> usize;
        pub fn read(key_ptr: *const u8, key_size: usize, value_ptr: *mut u8, value_size: usize);
        pub fn write(key_ptr: *const u8, key_size: usize, value_ptr: *const u8, value_size: usize);
        pub fn add(key_ptr: *const u8, key_size: usize, value_ptr: *const u8, value_size: usize);
        pub fn new_uref(key_ptr: *mut u8);
    }
}

pub mod ext {
    use super::alloc::alloc::{Alloc, Global};
    use super::alloc::vec::Vec;
    use super::ext_ffi;
    use crate::key::{Key, UREF_SIZE};
    use crate::bytesrepr::BytesRepr;
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
        Value::from_bytes(value_bytes).unwrap().0
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
        Key::from_bytes(slice).unwrap().0
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
