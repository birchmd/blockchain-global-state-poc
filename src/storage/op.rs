use std::ops::Add;

#[derive(PartialEq, Eq, Debug)]
pub enum Op {
    Read,
    Write,
    Add,
    NoOp,
}

use self::Op::*;

impl Add for Op {
    type Output = Op;

    fn add(self, other: Op) -> Op {
        match (self, other) {
            (a, NoOp) => a,
            (NoOp, b) => b,
            (Read, Read) => Read,
            (Add, Add) => Add,
            _ => Write,
        }
    }
}
