use quote::ToTokens;
use syn::{Attribute, Error, Type};

pub enum ProtoCodecEndianness {
    LE,
    BE,
    VAR,
}

#[derive(Default)]
pub struct ProtoCodecFlags {
    pub endianness: Option<ProtoCodecEndianness>,
    pub vec_endianness: Option<ProtoCodecEndianness>,
    pub vec_repr: Option<Type>,
    pub enum_endianness: Option<ProtoCodecEndianness>,
    pub enum_repr: Option<Type>,
    pub nbt: bool,
    pub str: bool,
}

macro_rules! endianness {
    ($name:ident, $attr:ident, $flags:ident) => {
        $attr.parse_nested_meta(|meta| {
            // We disallow overwriting it later down the line
            if let Some(_) = $flags.$name {
                return Err(meta.error(format!("{} can not be overwritten", stringify!($name))));
            }

            // #[endianness(le)]
            if meta.path.is_ident("le") {
                $flags.$name = Some(ProtoCodecEndianness::LE);
                return Ok(());
            }

            // #[endianness(be)]
            if meta.path.is_ident("be") {
                $flags.$name = Some(ProtoCodecEndianness::BE);
                return Ok(());
            }

            // #[endianness(var)]
            if meta.path.is_ident("var") {
                $flags.$name = Some(ProtoCodecEndianness::VAR);
                return Ok(());
            }

            Err(meta.error(format!("unrecognized {}", stringify!($name))))
        })?;
    };
}

macro_rules! repr {
    ($name:ident, $attr:ident, $flags:ident) => {
        $attr.parse_nested_meta(|meta| {
            // We disallow overwriting it later down the line
            if let Some(_) = $flags.$name {
                return Err(meta.error(format!("{} can not be overwritten", stringify!($name))));
            };

            $flags.$name = if let Some(v) = meta.path.get_ident() {
                Some(Type::Verbatim(v.clone().to_token_stream()))
            } else {
                return Err(meta.error(format!("Missing integer type in {}", stringify!($name))));
            };

            Ok(())
        })?;
    };
}

pub fn get_attrs(attrs: &[Attribute]) -> Result<ProtoCodecFlags, Error> {
    let mut flags = ProtoCodecFlags::default();

    for attr in attrs {
        if attr.path().is_ident("endianness") {
            endianness!(endianness, attr, flags);
            continue;
        }

        if attr.path().is_ident("vec_endianness") {
            endianness!(vec_endianness, attr, flags);
            continue;
        }

        if attr.path().is_ident("vec_repr") {
            repr!(vec_repr, attr, flags);
            continue;
        }

        if attr.path().is_ident("enum_endianness") {
            endianness!(enum_endianness, attr, flags);
            continue;
        }

        if attr.path().is_ident("enum_repr") {
            repr!(enum_repr, attr, flags);
            continue;
        }

        if attr.path().is_ident("nbt") {
            flags.nbt = true;
            continue;
        }

        if attr.path().is_ident("str") {
            flags.str = true;
            continue;
        }
    }

    Ok(flags)
}
