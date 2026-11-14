use crate::error::ProtoCodecError;

#[derive(Debug, Clone)]
pub enum SubClientID {
    PrimaryClient,
    Client2,
    Client3,
    Client4,
}

macro_rules! impl_sub_client {
    ($int:ty) => {
        impl TryFrom<$int> for SubClientID {
            type Error = ProtoCodecError;

            fn try_from(value: $int) -> Result<Self, Self::Error> {
                match value {
                    0 => Ok(SubClientID::PrimaryClient),
                    1 => Ok(SubClientID::Client2),
                    2 => Ok(SubClientID::Client3),
                    3 => Ok(SubClientID::Client4),
                    other => Err(ProtoCodecError::InvalidEnumID(
                        format!("{other:?}"),
                        "SubClientID",
                    )),
                }
            }
        }

        impl From<SubClientID> for $int {
            fn from(value: SubClientID) -> Self {
                match value {
                    SubClientID::PrimaryClient => 0,
                    SubClientID::Client2 => 1,
                    SubClientID::Client3 => 2,
                    SubClientID::Client4 => 3,
                }
            }
        }
    };
}

impl_sub_client!(u8);
impl_sub_client!(i8);
impl_sub_client!(u16);
impl_sub_client!(i16);
impl_sub_client!(u32);
impl_sub_client!(i32);
impl_sub_client!(u64);
impl_sub_client!(i64);
impl_sub_client!(u128);
impl_sub_client!(i128);
