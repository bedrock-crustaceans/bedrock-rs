use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i32)]
#[enum_endianness(be)]
#[repr(i32)]
pub enum PlayStatus {
    LoginSuccess = 0,
    LoginFailedClientOld = 1,
    LoginFailedServerOld = 2,
    PlayerSpawn = 3,
    LoginFailedInvalidTenant = 4,
    LoginFailedEditionMismatchEduToVanilla = 5,
    LoginFailedEditionMismatchVanillaToEdu = 6,
    LoginFailedServerFullSubClient = 7,
    LoginFailedEditorMismatchEditorToVanilla = 8,
    LoginFailedEditorMismatchVanillaToEditor = 9,
}
