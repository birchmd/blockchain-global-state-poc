use std::collections::HashMap;

pub mod key;
pub mod op;
pub mod transform;
mod utils;
pub mod value;

use self::key::Key;
use self::op::Op;
use self::transform::Transform;
use self::utils::add;
use self::value::Value;

pub enum Error {
    KeyNotFound { key: Key },
    TypeMismatch { expected: String, found: String },
    Unknown,
}

pub trait GlobalState<T: TrackingCopy> {
    fn apply(&mut self, k: Key, t: Transform) -> Result<(), Error>;
    fn get(&self, k: &Key) -> Result<&Value, Error>;
    fn tracking_copy(&self) -> T;
}

pub type ExecutionEffect = (HashMap<Key, Op>, HashMap<Key, Transform>);

pub trait TrackingCopy {
    fn read(&mut self, k: Key) -> Result<&Value, Error>;
    fn write(&mut self, k: Key, v: Value) -> Result<(), Error>;
    fn add(&mut self, k: Key, v: Value) -> Result<(), Error>;
    fn effect(&self) -> ExecutionEffect;
}

pub struct InMemGS {
    store: HashMap<Key, Value>
}

impl GlobalState<InMemTC> for InMemGS {
    fn apply(&mut self, k: Key, t: Transform) -> Result<(), Error> {
        let maybe_curr = self.store.remove(&k);
        match maybe_curr {
            None => Err(Error::KeyNotFound{ key: k }),
            Some(curr) => {
                let new_value = t.apply(curr)?;
                let _ = self.store.insert(k, new_value);
                Ok(())
            }
        }
    }
    fn get(&self, k: &Key) -> Result<&Value, Error> {
        match self.store.get(k) {
            None => Err(Error::KeyNotFound{ key: *k }),
            Some(v) => Ok(v)
        }
    } 
    fn tracking_copy(&self) -> InMemTC {
        InMemTC {
            store: self.store.clone(), //TODO: make more efficient
            ops: HashMap::new(),
            fns: HashMap::new()
        }
    }
}

pub struct InMemTC {
    store: HashMap<Key, Value>,
    ops: HashMap<Key, Op>,
    fns: HashMap<Key, Transform>,
}

impl TrackingCopy for InMemTC {
    fn read(&mut self, k: Key) -> Result<&Value, Error> {
        let maybe_value = self.store.get(&k);
        match maybe_value {
            None => Err(Error::KeyNotFound { key: k }),
            Some(v) => {
                add(&mut self.ops, k, Op::Read);
                Ok(v)
            }
        }
    }
    fn write(&mut self, k: Key, v: Value) -> Result<(), Error> {
        let _ = self.store.insert(k, v.clone());
        add(&mut self.ops, k, Op::Write);
        add(&mut self.fns, k, Transform::Write(v));
        Ok(())
    }
    fn add(&mut self, k: Key, v: Value) -> Result<(), Error> {
        let expected = "Int32".to_string();
        match v {
            Value::Int32(i) => {
                let maybe_curr = self.store.remove(&k);
                match maybe_curr {
                    None => Err(Error::KeyNotFound { key: k }),
                    Some(curr) => match curr {
                        Value::Int32(j) => {
                            let _ = self.store.insert(k, Value::Int32(i + j));
                            add(&mut self.ops, k, Op::Add);
                            add(&mut self.fns, k, Transform::Add(j));
                            Ok(())
                        }
                        other => Err(Error::TypeMismatch {
                            expected,
                            found: other.type_string(),
                        }),
                    },
                }
            }
            other => Err(Error::TypeMismatch {
                expected,
                found: other.type_string(),
            }),
        }
    }
    fn effect(&self) -> ExecutionEffect {
        (self.ops.clone(), self.fns.clone())
    }
}
