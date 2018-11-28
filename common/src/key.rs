use super::alloc::alloc::{Alloc, Global};

#[repr(C)]
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum Key {
    Account([u8; 20]),
    Hash([u8; 32]),
    URef([u8; 32]), //TODO: more bytes?
}

impl Key {
    pub fn from_bytes(bytes: &[u8]) -> Key {
        unsafe {
            let ptr: *mut Key = Global.alloc_one().unwrap().as_ptr();
            let b_ptr = ptr as *mut u8;
            for i in 0..bytes.len() {
                core::ptr::write(b_ptr.offset(i as isize), bytes[i])
            }
            core::ptr::read(ptr)
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        let ptr = self as *const Self;
        let b_ptr = ptr as *const u8;
        let size = core::mem::size_of::<Self>();
        unsafe { core::slice::from_raw_parts(b_ptr, size) }
    }
}
