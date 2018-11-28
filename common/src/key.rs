use super::memio::MemIO;

#[repr(C)]
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum Key {
    Account([u8; 20]),
    Hash([u8; 32]),
    URef([u8; 32]), //TODO: more bytes?
}

impl MemIO for Key {}
