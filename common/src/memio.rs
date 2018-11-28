use super::alloc::alloc::{Alloc, Global};

pub trait MemIO: Sized {
    
    fn from_bytes(bytes: &[u8]) -> Self {
        unsafe {
            let ptr: *mut Self = Global.alloc_one().unwrap().as_ptr();
            let b_ptr = ptr as *mut u8;
            for i in 0..bytes.len() {
                core::ptr::write(b_ptr.offset(i as isize), bytes[i])
            }
            core::ptr::read(ptr)
        }
    }

    fn as_bytes(&self) -> &[u8] {
        let ptr = self as *const Self;
        let b_ptr = ptr as *const u8;
        let size = core::mem::size_of::<Self>();
        unsafe { core::slice::from_raw_parts(b_ptr, size) }
    }

}
