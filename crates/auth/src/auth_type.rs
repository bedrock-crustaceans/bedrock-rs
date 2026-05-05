use serde_repr::Deserialize_repr;

#[derive(Deserialize_repr, Clone, Debug)]
#[repr(u8)]
pub enum AuthType {
    Online = 0,
    Guest = 1,
    Offline = 2,
}
