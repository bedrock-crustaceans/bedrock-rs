//! What the derive reads from a field's type, by its spelling: whether it is an `Option`, and
//! what it holds below `Option`, `Vec`, `Box` and maps.

use syn::{GenericArgument, PathArguments, Type};

/// The last segment's name and its type arguments.
fn last(ty: &Type) -> Option<(String, Vec<&Type>)> {
    let Type::Path(path) = ty else {
        return None;
    };
    if path.qself.is_some() {
        return None;
    }
    let segment = path.path.segments.last()?;
    let args = match &segment.arguments {
        PathArguments::AngleBracketed(args) => args
            .args
            .iter()
            .filter_map(|arg| match arg {
                GenericArgument::Type(ty) => Some(ty),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    };
    Some((segment.ident.to_string(), args))
}

/// `T` of `Option<T>`.
pub(crate) fn option_inner(ty: &Type) -> Option<&Type> {
    match last(ty)? {
        (name, args) if name == "Option" && args.len() == 1 => Some(args[0]),
        _ => None,
    }
}

/// The type below every `Option`, `Vec`, `Box` and map (`IndexMap`, `HashMap`, `BTreeMap`: the
/// value type) around it.
pub(crate) fn innermost(ty: &Type) -> &Type {
    match last(ty) {
        Some((name, args))
            if matches!(name.as_str(), "Option" | "Vec" | "Box") && args.len() == 1 =>
        {
            innermost(args[0])
        }
        Some((name, args))
            if matches!(name.as_str(), "IndexMap" | "HashMap" | "BTreeMap") && args.len() >= 2 =>
        {
            innermost(args[1])
        }
        _ => ty,
    }
}

/// The last segment's name of `ty` (no arguments), if it is a path.
pub(crate) fn name(ty: &Type) -> Option<String> {
    last(ty).map(|(name, _)| name)
}

/// Whether the field holds Molang values the visitor takes as a slot: a `Molang`,
/// `MolangSource` or `MolangList` (or molang's `MolangValueRepr`) below its containers.
pub(crate) fn is_molang_slot(ty: &Type) -> bool {
    name(innermost(ty)).is_some_and(|name| {
        matches!(
            name.as_str(),
            "Molang" | "MolangSource" | "MolangList" | "MolangValueRepr"
        )
    })
}

/// Whether the field holds event triggers below its containers.
pub(crate) fn is_event_trigger(ty: &Type) -> bool {
    name(innermost(ty)).is_some_and(|name| name == "EventTrigger")
}

/// Whether the field is a `String` or an `Option<String>`.
pub(crate) fn is_string(ty: &Type) -> bool {
    let ty = option_inner(ty).unwrap_or(ty);
    name(ty).is_some_and(|name| name == "String")
}
