#![no_std]
#![feature(alloc, allocator_api, core_intrinsics, lang_items, alloc_error_handler)]

extern crate alloc;
extern crate wee_alloc;

#[cfg(not(feature = "std"))]
#[global_allocator]
pub static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub mod key;
pub mod memio;
pub mod value;

mod ext_ffi {
    extern "C" {
        pub fn read(key_ptr: *const u8, value_ptr: *mut u8);
        pub fn write(key_ptr: *const u8, value_ptr: *const u8);
        pub fn add(key_ptr: *const u8, value_ptr: *const u8);
        pub fn new_uref(key_ptr: *mut u8);
    }
}

pub mod ext {
    use super::alloc::alloc::{Alloc, Global};
    use super::ext_ffi;
    use crate::key::Key;
    use crate::value::Value;

    pub fn read(key: &Key) -> Value {
        let key_ptr = key as *const Key;
        let value_ptr: *mut Value = Global.alloc_one().unwrap().as_ptr();
        unsafe {
            ext_ffi::read(key_ptr as *const u8, value_ptr as *mut u8);
            core::ptr::read(value_ptr)
        }
    }

    pub fn write(key: &Key, value: &Value) {
        let key_ptr = key as *const Key;
        let value_ptr = value as *const Value;
        unsafe {
            ext_ffi::write(key_ptr as *const u8, value_ptr as *const u8);
        }
    }

    pub fn add(key: &Key, value: &Value) {
        let key_ptr = key as *const Key;
        let value_ptr = value as *const Value;
        unsafe {
            ext_ffi::add(key_ptr as *const u8, value_ptr as *const u8);
        }
    }
    
    pub fn new_uref() -> Key {
        let key_ptr: *mut Key = Global.alloc_one().unwrap().as_ptr();
        unsafe {
            ext_ffi::new_uref(key_ptr as *mut u8);
            core::ptr::read(key_ptr)
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
