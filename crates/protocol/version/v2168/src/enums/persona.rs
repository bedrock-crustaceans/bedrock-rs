use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u32)]
#[enum_endianness(var)]
#[repr(u32)]
pub enum AnimatedTextureType {
    None = 0,
    Face = 1,
    Body32x32 = 2,
    Body128x128 = 3,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u32)]
#[enum_endianness(var)]
#[repr(u32)]
pub enum AnimationExpression {
    Linear = 0,
    Blinking = 1,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u8)]
#[repr(u8)]
pub enum ArmSizeType {
    Slim = 0,
    Wide = 1,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i32)]
#[enum_endianness(le)]
#[repr(i32)]
pub enum PersonaPieceType {
    Skeleton = 0,
    Body = 1,
    Skin = 2,
    Bottom = 3,
    Feet = 4,
    Dress = 5,
    Top = 6,
    HighPants = 7,
    Hands = 8,
    Outerwear = 9,
    FacialHair = 10,
    Mouth = 11,
    Eyes = 12,
    Hair = 13,
    Hood = 14,
    Back = 15,
    FaceAccessory = 16,
    Head = 17,
    Legs = 18,
    LeftLeg = 19,
    RightLeg = 20,
    Arms = 21,
    LeftArm = 22,
    RightArm = 23,
    Capes = 24,
    ClassicSkin = 25,
    Emote = 26,
}
