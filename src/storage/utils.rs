use std::collections::HashMap;
use std::hash::Hash;

pub fn add<K, V>(map: &mut HashMap<K, V>, k: K, v: V)
where
    K: Eq + Hash,
    V: Monoid,
{
    let maybe_curr = map.remove(&k);
    match maybe_curr {
        None => {
            let _ = map.insert(k, v);
        }
        Some(curr) => {
            let _ = map.insert(k, curr.combine(v) );
        }
    }
}

pub trait Semigroup {
    fn zero() -> Self;
}

pub trait Monoid: Semigroup {
    fn combine(self, other: Self) -> Self;
}