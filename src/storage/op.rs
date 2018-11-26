use std::fmt;
use super::utils::{Semigroup, Monoid};

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Op {
    Read,
    Write,
    Add,
    NoOp,
}

use self::Op::*;

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Semigroup for Op {
    fn zero() -> Self {
        Op::NoOp
    }
}

impl Monoid for Op {
    fn combine(self, other: Self) -> Self {
        match (self, other) {
            (a, NoOp) => a,
            (NoOp, b) => b,
            (Read, Read) => Read,
            (Add, Add) => Add,
            _ => Write,
        }
    }
}