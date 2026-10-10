use quote::ToTokens;
use syn::{Attribute, DataEnum, Error, Fields, GenericArgument, PathArguments, Type, Variant};

#[derive(Clone)]
pub enum ProtoCodecEndianness {
    Le,
    Be,
    Var,
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
                $flags.$name = Some(ProtoCodecEndianness::Le);
                return Ok(());
            }

            // #[endianness(be)]
            if meta.path.is_ident("be") {
                $flags.$name = Some(ProtoCodecEndianness::Be);
                return Ok(());
            }

            // #[endianness(var)]
            if meta.path.is_ident("var") {
                $flags.$name = Some(ProtoCodecEndianness::Var);
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

pub fn enum_fallback(data_enum: &DataEnum) -> syn::Result<Option<&Variant>> {
    let mut fallbacks = data_enum
        .variants
        .iter()
        .filter(|variant| is_enum_fallback(variant));

    let Some(fallback) = fallbacks.next() else {
        return Ok(None);
    };

    if let Some(extra) = fallbacks.next() {
        return Err(Error::new_spanned(
            extra,
            "only one variant can be marked #[enum_fallback]",
        ));
    }

    match &fallback.fields {
        Fields::Unnamed(fields) if fields.unnamed.len() == 1 => Ok(Some(fallback)),
        _ => Err(Error::new_spanned(
            fallback,
            "#[enum_fallback] requires a tuple variant with a single field",
        )),
    }
}

pub fn is_enum_fallback(variant: &Variant) -> bool {
    variant
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("enum_fallback"))
}

pub fn extract_inner_type_from_vec(ty: &Type) -> Option<&Type> {
    if let Type::Path(type_path) = ty
        && let Some(last_segment) = type_path.path.segments.last()
        && last_segment.ident == "Vec"
        && let PathArguments::AngleBracketed(ref generics) = last_segment.arguments
        && generics.args.len() == 1
        && let Some(GenericArgument::Type(inner_type)) = generics.args.first()
    {
        return Some(inner_type);
    }

    None
}
