//! Type descriptors in the form consumers use: [`TypeDescriptor::lower`] turns the written
//! [`RawTypeDesc`] into one enum per kind of type.

use super::model::script::{FromModule, RawTypeDesc};
use super::{MetaError, PrivilegeList};

/// A lowered type descriptor: what the type is, whether it can throw, and the TypeScript
/// overlay flags.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct TypeDescriptor {
    /// What the type is.
    pub kind: TypeKind,
    /// Whether a call returning it may throw, and what.
    pub errors: Errorability,
    /// The TypeScript overlay flags (all `false` for descriptors the game wrote itself).
    pub overlay: TypeOverlay,
}

/// Whether a call may throw (`is_errorable`, `error_types`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Errorability {
    /// `is_errorable: false`.
    #[default]
    No,
    /// `is_errorable: true` without `error_types`: it may throw, the errors are not listed.
    Unlisted,
    /// `is_errorable: true` with `error_types`.
    Listed(Vec<TypeDescriptor>),
}

/// What a descriptor names.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum TypeKind {
    /// A primitive (`string`, `int32`, …).
    Prim(Prim),
    /// A declared class, interface, enum, error or alias (`is_bind_type: true`), and the native
    /// module it comes from when that is another module.
    Bind {
        /// The declared name.
        name: String,
        /// `from_module`: the declaring module of a foreign type.
        from: Option<FromModule>,
    },
    /// `optional`: the value or `undefined`.
    Optional(Box<TypeDescriptor>),
    /// `array`.
    Array(Box<TypeDescriptor>),
    /// `map`.
    Map {
        /// `key_type`.
        key: Box<TypeDescriptor>,
        /// `value_type`.
        value: Box<TypeDescriptor>,
    },
    /// `variant`: one of several types.
    Variant(Vec<TypeDescriptor>),
    /// `promise`.
    Promise(Box<TypeDescriptor>),
    /// `closure`: a callback.
    Closure {
        /// `argument_types`.
        args: Vec<TypeDescriptor>,
        /// `return_type`.
        ret: Box<TypeDescriptor>,
        /// `call_privilege`: the privilege the callback runs with.
        privilege: Option<PrivilegeList>,
    },
    /// `generator`.
    Generator {
        /// `yield_type`.
        yield_: Box<TypeDescriptor>,
        /// `return_type`.
        ret: Box<TypeDescriptor>,
        /// `next_type`.
        next: Box<TypeDescriptor>,
    },
    /// A TypeScript generic instantiation (`generic_base` with `generic_types`).
    Generic {
        /// `generic_base`.
        base: Box<TypeDescriptor>,
        /// `generic_types`.
        args: Vec<TypeDescriptor>,
    },
    /// Any other name: TypeScript text, a type parameter (`T`), a `minecraftvanilladata.*`
    /// reference. Opaque to this crate.
    Opaque(String),
}

/// A primitive type name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Prim {
    /// `string`.
    String,
    /// `boolean`.
    Boolean,
    /// `int8`.
    Int8,
    /// `int16`.
    Int16,
    /// `int32`.
    Int32,
    /// `int64`.
    Int64,
    /// `uint8`.
    Uint8,
    /// `uint16`.
    Uint16,
    /// `uint32`.
    Uint32,
    /// `uint64`.
    Uint64,
    /// `float`.
    Float,
    /// `double`.
    Double,
    /// `undefined`.
    Undefined,
    /// `void`.
    Void,
    /// `unknown`.
    Unknown,
    /// `any`.
    Any,
    /// `never`.
    Never,
    /// `number`.
    Number,
    /// `Error`: JavaScript's error class (lower-case `error` is opaque).
    JsError,
}

impl Prim {
    /// The primitive written `name`, compared exactly.
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "string" => Self::String,
            "boolean" => Self::Boolean,
            "int8" => Self::Int8,
            "int16" => Self::Int16,
            "int32" => Self::Int32,
            "int64" => Self::Int64,
            "uint8" => Self::Uint8,
            "uint16" => Self::Uint16,
            "uint32" => Self::Uint32,
            "uint64" => Self::Uint64,
            "float" => Self::Float,
            "double" => Self::Double,
            "undefined" => Self::Undefined,
            "void" => Self::Void,
            "unknown" => Self::Unknown,
            "any" => Self::Any,
            "never" => Self::Never,
            "number" => Self::Number,
            "Error" => Self::JsError,
            _ => return None,
        })
    }
}

/// The TypeScript overlay flags of a descriptor (`is_core`, `is_class`, `is_interface`,
/// `is_enum`, `is_type`, `is_optional`); an absent flag is `false`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[allow(clippy::struct_excessive_bools)] // one flag per JSON member
#[non_exhaustive]
pub struct TypeOverlay {
    /// `is_core`.
    pub is_core: bool,
    /// `is_class`.
    pub is_class: bool,
    /// `is_interface`.
    pub is_interface: bool,
    /// `is_enum`.
    pub is_enum: bool,
    /// `is_type`.
    pub is_type: bool,
    /// `is_optional`.
    pub is_optional: bool,
}

impl TypeDescriptor {
    /// Lowers a written descriptor. A composite tag (`optional`, `array`, `map`, `variant`,
    /// `promise`, `closure`, `generator`) needs its payload members and no other; a payload
    /// member under any other name, or on a declared type, is an error. `generic_base` makes a
    /// [`TypeKind::Generic`] whatever the name; `is_bind_type` a [`TypeKind::Bind`]; a name in
    /// the primitive table a [`TypeKind::Prim`]; anything else is [`TypeKind::Opaque`].
    /// `valid_range` is dropped.
    ///
    /// # Errors
    ///
    /// Returns a [`MetaError`] when a composite tag lacks one of its payload members
    /// ([`MetaError::MissingPayload`]), a descriptor carries a payload member its name does not
    /// take ([`MetaError::UnexpectedPayload`]), or `error_types` is empty or given without
    /// `is_errorable` ([`MetaError::ErrorTypes`]).
    pub fn lower(raw: &RawTypeDesc) -> Result<Self, MetaError> {
        check_payload(raw)?;
        let kind = if let Some(base) = &raw.generic_base {
            TypeKind::Generic {
                base: Box::new(Self::lower(base)?),
                args: lower_all(raw.generic_types.as_deref().unwrap_or_default())?,
            }
        } else if raw.is_bind_type {
            TypeKind::Bind {
                name: raw.name.clone(),
                from: raw.from_module.clone(),
            }
        } else {
            composite(raw)?.unwrap_or_else(|| {
                Prim::from_name(&raw.name)
                    .map_or_else(|| TypeKind::Opaque(raw.name.clone()), TypeKind::Prim)
            })
        };
        let errors = match (raw.is_errorable, &raw.error_types) {
            (false, None) => Errorability::No,
            (true, None) => Errorability::Unlisted,
            (true, Some(list)) if !list.is_empty() => Errorability::Listed(lower_all(list)?),
            _ => return Err(MetaError::ErrorTypes),
        };
        let flag = |value: Option<bool>| value.unwrap_or(false);
        let overlay = TypeOverlay {
            is_core: flag(raw.is_core),
            is_class: flag(raw.is_class),
            is_interface: flag(raw.is_interface),
            is_enum: flag(raw.is_enum),
            is_type: flag(raw.is_type),
            is_optional: flag(raw.is_optional),
        };
        Ok(Self {
            kind,
            errors,
            overlay,
        })
    }
}

/// The composite tags and the payload members each takes.
const COMPOSITES: [(&str, &[&str]); 7] = [
    ("optional", &["optional_type"]),
    ("array", &["element_type"]),
    ("map", &["key_type", "value_type"]),
    ("variant", &["variant_types"]),
    ("promise", &["promise_type"]),
    ("closure", &["closure_type"]),
    ("generator", &["generator_type"]),
];

/// The payload members `raw` carries, in the order of [`COMPOSITES`].
fn payload_keys(raw: &RawTypeDesc) -> Vec<&'static str> {
    [
        ("optional_type", raw.optional_type.is_some()),
        ("element_type", raw.element_type.is_some()),
        ("key_type", raw.key_type.is_some()),
        ("value_type", raw.value_type.is_some()),
        ("variant_types", raw.variant_types.is_some()),
        ("promise_type", raw.promise_type.is_some()),
        ("closure_type", raw.closure_type.is_some()),
        ("generator_type", raw.generator_type.is_some()),
    ]
    .into_iter()
    .filter_map(|(key, present)| present.then_some(key))
    .collect()
}

/// Rules 1–3: a composite tag has exactly its payload members; any other descriptor, and a
/// declared type whatever its name, has none.
fn check_payload(raw: &RawTypeDesc) -> Result<(), MetaError> {
    let present = payload_keys(raw);
    let wanted: &[&str] = match COMPOSITES.iter().find(|(tag, _)| *tag == raw.name) {
        Some(_) if raw.is_bind_type => &[],
        Some((_, keys)) => keys,
        None => &[],
    };
    if let Some(key) = present.iter().find(|key| !wanted.contains(key)) {
        return Err(MetaError::UnexpectedPayload { key });
    }
    if let Some((tag, _)) = COMPOSITES.iter().find(|(tag, _)| *tag == raw.name)
        && let Some(key) = wanted.iter().find(|key| !present.contains(key))
    {
        return Err(MetaError::MissingPayload { tag, key });
    }
    Ok(())
}

/// The kind of a composite descriptor whose payload [`check_payload`] accepted; `None` for any
/// other name.
fn composite(raw: &RawTypeDesc) -> Result<Option<TypeKind>, MetaError> {
    let Some(&(tag, _)) = COMPOSITES.iter().find(|(tag, _)| *tag == raw.name) else {
        return Ok(None);
    };
    let missing = |key| MetaError::MissingPayload { tag, key };
    let lower = |inner: &Option<Box<RawTypeDesc>>, key| -> Result<Box<TypeDescriptor>, MetaError> {
        let inner = inner.as_deref().ok_or_else(|| missing(key))?;
        TypeDescriptor::lower(inner).map(Box::new)
    };
    Ok(Some(match tag {
        "optional" => TypeKind::Optional(lower(&raw.optional_type, "optional_type")?),
        "array" => TypeKind::Array(lower(&raw.element_type, "element_type")?),
        "map" => TypeKind::Map {
            key: lower(&raw.key_type, "key_type")?,
            value: lower(&raw.value_type, "value_type")?,
        },
        "variant" => TypeKind::Variant(lower_all(
            raw.variant_types
                .as_deref()
                .ok_or_else(|| missing("variant_types"))?,
        )?),
        "promise" => TypeKind::Promise(lower(&raw.promise_type, "promise_type")?),
        "closure" => {
            let closure = raw
                .closure_type
                .as_deref()
                .ok_or_else(|| missing("closure_type"))?;
            TypeKind::Closure {
                args: lower_all(&closure.argument_types)?,
                ret: Box::new(TypeDescriptor::lower(&closure.return_type)?),
                privilege: closure
                    .call_privilege
                    .as_ref()
                    .map(|name| PrivilegeList::from_names(std::slice::from_ref(name)))
                    .transpose()?,
            }
        }
        _ => {
            let generator = raw
                .generator_type
                .as_deref()
                .ok_or_else(|| missing("generator_type"))?;
            TypeKind::Generator {
                yield_: Box::new(TypeDescriptor::lower(&generator.yield_type)?),
                ret: Box::new(TypeDescriptor::lower(&generator.return_type)?),
                next: Box::new(TypeDescriptor::lower(&generator.next_type)?),
            }
        }
    }))
}

fn lower_all(list: &[RawTypeDesc]) -> Result<Vec<TypeDescriptor>, MetaError> {
    list.iter().map(TypeDescriptor::lower).collect()
}
