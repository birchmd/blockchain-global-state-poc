use super::alloc::alloc::{Alloc, Global};
use super::alloc::string::String;
use core::ops::Deref;

#[repr(C)]
#[derive(Debug)]
pub struct Array<T> {
    elems: *mut T,
    pub size: usize,
}
impl<T> Deref for Array<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.elems, self.size) }
    }
}

impl<T: PartialEq> PartialEq for Array<T> {
    fn eq(&self, other: &Array<T>) -> bool {
        if self.size != other.size {
            false
        } else {
            self.iter().zip(other.iter()).all(|(x, y)| x == y)
        }
    }
}
impl<T: Eq> Eq for Array<T> {}
impl<T: Clone> Clone for Array<T> {
    fn clone(&self) -> Self {
        unsafe {
            let ptr: *mut T = Global.alloc_array(self.size).unwrap().as_ptr();

            for i in 0..self.size {
                let t: T = (*self.elems.offset(i as isize)).clone();
                core::ptr::write(ptr.offset(i as isize), t);
            }

            Array {
                elems: ptr,
                size: self.size,
            }
        }
    }
}

pub type CharArray = Array<u8>; //ascii-encoded string

#[repr(C)]
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Value {
    Int32(i32),
    ByteArray(Array<u8>),
    ListInt32(Array<i32>),
    String(CharArray),
    Acct(Account),
}

#[repr(C)]
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Account {
    public_key: [u8; 32],
    nonce: u64,
    known_urefs: Array<super::key::Key>,
}

use self::Value::*;

impl Value {
    pub fn from_bytes(bytes: &[u8]) -> Value {
        unsafe {
            let ptr: *mut Value = Global.alloc_one().unwrap().as_ptr();
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

    pub fn type_string(&self) -> String {
        match self {
            Int32(_) => String::from("Int32"),
            ListInt32(_) => String::from("List[Int32]"),
            String(_) => String::from("String"),
            ByteArray(_) => String::from("ByteArray"),
            Acct(_) => String::from("Account"),
        }
    }

    pub fn as_account(&self) -> &Account {
        match self {
            Acct(a) => a,
            _ => panic!("Not an account: {:?}", self),
        }
    }
}

impl Account {
    pub fn urefs(&self) -> &[super::key::Key] {
        &self.known_urefs
    }
}
