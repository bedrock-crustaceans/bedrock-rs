use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i32)]
#[enum_endianness(le)]
#[repr(i32)]
pub enum MolangVersion {
    Invalid = -1,
    BeforeVersioning = 0,
    Initial = 1,
    FixedItemRemainingUseDurationQuery = 2,
    ExpressionErrorMessages = 3,
    UnexpectedOperatorErrors = 4,
    ConditionalOperatorAssociativity = 5,
    ComparisonAndLogicalOperatorPrecedence = 6,
    DivideByNegativeValue = 7,
    FixedCapeFlapAmountQuery = 8,
    QueryBlockPropertyRenamedToState = 9,
    DeprecateOldBlockQueryNames = 10,
    DeprecatedSnifferAndCamelQueries = 11,
    LeafSupportingInFirstSolidBlockBelow = 12,
    NumValidVersions = 13,
}

impl MolangVersion {
    pub const LATEST: MolangVersion = MolangVersion::LeafSupportingInFirstSolidBlockBelow; // TODO: NumValidVersions - 1 (error)
    pub const HARDCODED_MOLANG: MolangVersion = MolangVersion::LATEST;
}