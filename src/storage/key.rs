#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum Key {
    Address([u8; 20]),
    Hash([u8; 32]),
    URef([u8; 32]), //TODO: more bytes?
}
