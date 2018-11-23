use super::value::Value;
use std::ops::Add;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Transform {
    Identity,
    Write(Value),
    Add(i32),
}

use self::Transform::*;

impl Transform {
    pub fn apply(&self, v: Value) -> Result<Value, super::Error> {
        match self {
            Identity => Ok(v),
            Write(w) => Ok(w.clone()),
            Add(i) => match v {
                Value::Int32(j) => Ok(Value::Int32(i + j)),
                other => {
                    let expected = "Int32".to_string();
                    Err(super::Error::TypeMismatch{ expected, found: other.type_string() })
                },
            },
        }
    }
}

impl Add for Transform {
    type Output = Transform;

    fn add(self, other: Transform) -> Transform {
        match (self, other) {
            (a, Identity) => a,
            (Identity, b) => b,
            (_, b @ Write(_)) => b,
            (Add(i), Add(j)) => Add(i + j),
            (Write(v), Add(j)) => match v {
                Value::Int32(i) => Write(Value::Int32(i + j)),
                other => Write(other),
            },
        }
    }
}
