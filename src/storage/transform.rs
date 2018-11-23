use super::value::Value;
use std::ops::Add;

#[derive(PartialEq, Eq, Debug)]
pub enum Transform {
    Identity,
    Write(Value),
    Add(i32),
}

use self::Transform::*;

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
