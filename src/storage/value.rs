use std::collections::{HashMap, HashSet};
use std::fmt;
use std::string::String;

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Value {
    Int32(i32),
    ByteArray(Vec<u8>),
    ListInt32(Vec<i32>),
    String(String),
    Acct(Account),
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Account {
    public_key: [u8; 32],
    nonce: u64,
    known_urefs: HashMap<String, super::Key>,
}

use self::Value::*;

impl Value {
    pub fn type_string(&self) -> String {
        match self {
            Int32(_) => "Int32".to_string(),
            ListInt32(_) => "List[Int32]".to_string(),
            String(_) => "String".to_string(),
            ByteArray(_) => "ByteArray".to_string(),
            Acct(_) => "Account".to_string(),
        }
    }

    pub fn as_account(&self) -> &Account {
        match self {
            Acct(a) => a,
            _ => panic!("Not an account: {}", self),
        }
    }
}

impl Account {
    pub fn urefs(&self) -> HashSet<super::Key> {
        let mut result = HashSet::new();
        for r in self.known_urefs.values() {
            result.insert(r.clone());
        }
        result
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
