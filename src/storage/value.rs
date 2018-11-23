#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Value {
    Int32(i32),
    ListInt32(Vec<i32>),
    String(std::string::String),
    ByteArray(Vec<u8>),
}
