use std::string::String;

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Value {
    Int32(i32),
    ListInt32(Vec<i32>),
    String(String),
    ByteArray(Vec<u8>),
}

use self::Value::*;

impl Value {
    pub fn type_string(&self) -> String {
        match self {
            Int32(_) => "Int32".to_string(),
            ListInt32(_) => "List[Int32]".to_string(),
            String(_) => "String".to_string(),
            ByteArray(_) => "ByteArray".to_string(),
        }
    }
}
