use super::alloc::string::{self, String};
use super::alloc::vec::Vec;
use super::key::Key;
use super::bytesrepr::{BytesRepr, Error};

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Value {
    Int32(i32),
    ByteArray(Vec<u8>),
    ListInt32(Vec<i32>),
    String(string::String),
    Acct(Account),
}

const INT32_ID: u8 = 0;
const BYTEARRAY_ID: u8 = 1;
const LISTINT32_ID: u8 = 2;
const STRING_ID: u8 = 3;
const ACCT_ID: u8 = 4;

use self::Value::*;

impl BytesRepr for Value {
    fn to_bytes(&self) -> Vec<u8>  {
        match self {
            Int32(i) => {
                let mut result = Vec::with_capacity(5);
                result.push(INT32_ID);
                result.extend(i.to_bytes());
                result
            }

            ByteArray(arr) => {
                let mut result = Vec::new();
                result.push(BYTEARRAY_ID);
                result.extend(arr.to_bytes());
                result
            }
            ListInt32(arr) => {
                let mut result = Vec::new();
                result.push(LISTINT32_ID);
                result.extend(arr.to_bytes());
                result
            }
            String(s) => {
                let mut result = Vec::new();
                result.push(STRING_ID);
                result.extend(s.to_bytes());
                result
            }
            Acct(a) => {
                let mut result = Vec::new();
                result.push(ACCT_ID);
                result.extend(a.to_bytes());
                result
            }
        }
    }

    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let (id, rest): (u8, &[u8]) = BytesRepr::from_bytes(bytes)?;
        match id {
            INT32_ID => {
                let (i, rem): (i32, &[u8]) = BytesRepr::from_bytes(rest)?;
                Ok((Int32(i), rem))
            }
            BYTEARRAY_ID => {
                let (arr, rem): (Vec<u8>, &[u8]) = BytesRepr::from_bytes(rest)?;
                Ok((ByteArray(arr), rem))
            }
            LISTINT32_ID => {
                let (arr, rem): (Vec<i32>, &[u8]) = BytesRepr::from_bytes(rest)?;
                Ok((ListInt32(arr), rem))
            }
            STRING_ID => {
                let (s, rem): (String, &[u8]) = BytesRepr::from_bytes(rest)?;
                Ok((String(s), rem))
            }
            ACCT_ID => {
                let (a, rem): (Account, &[u8]) = BytesRepr::from_bytes(rest)?;
                Ok((Acct(a), rem))
            }
            _ => Err(Error::FormattingError),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Account {
    public_key: [u8; 32],
    nonce: u64,
    known_urefs: Vec<Key>,
}

impl BytesRepr for Account {
    fn to_bytes(&self) -> Vec<u8>  {
        let mut result = Vec::new();
        result.extend(&self.public_key);
        result.extend(self.nonce.to_bytes());
        result.extend(self.known_urefs.to_bytes());
        result
    }
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let (public_key, rem1): ([u8; 32], &[u8]) = BytesRepr::from_bytes(bytes)?;
        let (nonce, rem2): (u64, &[u8]) = BytesRepr::from_bytes(rem1)?;
        let (known_urefs, rem3): (Vec<Key>, &[u8]) = BytesRepr::from_bytes(rem2)?;
        Ok((Account{public_key, nonce, known_urefs}, rem3))
    }
}

impl Value {
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
    pub fn new(public_key: [u8; 32], nonce: u64, known_urefs: Vec<Key>) -> Account {
        Account {
            public_key,
            nonce,
            known_urefs,
        }
    }

    pub fn urefs(&self) -> &[Key] {
        &self.known_urefs
    }
}
