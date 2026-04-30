use crate::deserialize_bool;

#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Hash,
    serde_repr::Serialize_repr,
    serde_repr::Deserialize_repr,
)]
#[repr(i8)]
pub enum PistonState {
    Retracted = 0,
    Pushing = 1,
    Extended = 2,
    Pulling = 3,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum PistonProgress {
    None,
    Half,
    Full,
}

impl serde::Serialize for PistonProgress {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let val = match self {
            Self::None => 0.0,
            Self::Half => 0.5,
            Self::Full => 1.0,
        };

        serializer.serialize_f32(val)
    }
}

impl<'de> serde::Deserialize<'de> for PistonProgress {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let v = f32::deserialize(deserializer)?;
        Ok(match v {
            0.0 => Self::None,
            0.5 => Self::Half,
            1.0 => Self::Full,
            _ => {
                return Err(serde::de::Error::invalid_value(
                    serde::de::Unexpected::Float(v as f64),
                    &"floating point `0.0`, `0.5` or `1.0`",
                ));
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct PistonArm {
    /// Not sure how this is actually stored. Will need to create a custom type for this for clarity.
    pub attached_blocks: Vec<i32>,
    /// Same for this field.
    pub break_blocks: Vec<i32>,
    pub last_progress: PistonProgress,
    pub new_state: PistonState,
    pub progress: PistonProgress,
    pub state: PistonState,
    #[serde(deserialize_with = "deserialize_bool")]
    pub sticky: bool,
}
