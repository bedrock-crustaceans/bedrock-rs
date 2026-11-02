use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Debug, Clone, Eq, PartialEq, Copy, Hash)]
#[enum_repr(i32)]
#[enum_endianness(var)]
pub enum Dimension {
    Overworld = 0,
    Nether = 1,
    End = 2,
    Undefined = 3,
}

impl From<i32> for Dimension {
    fn from(value: i32) -> Self {
        match value {
            0 => Self::Overworld,
            1 => Self::Nether,
            2 => Self::End,
            _ => Self::Undefined,
        }
    }
}

impl From<Dimension> for i32 {
    fn from(value: Dimension) -> i32 {
        match value {
            Dimension::Overworld => 0,
            Dimension::Nether => 1,
            Dimension::End => 2,
            Dimension::Undefined => 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_dimension_round_trips_through_its_id() {
        for dimension in [
            Dimension::Overworld,
            Dimension::Nether,
            Dimension::End,
            Dimension::Undefined,
        ] {
            assert_eq!(Dimension::from(i32::from(dimension)), dimension);
        }
    }

    #[test]
    fn an_unknown_dimension_id_is_undefined() {
        assert_eq!(Dimension::from(4), Dimension::Undefined);
        assert_eq!(Dimension::from(-1), Dimension::Undefined);
    }
}
