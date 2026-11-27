//! The writer of a typed value, generic over its facet `Shape`: [`write_value`], the twin of
//! [`read_value`](super::read_value).
//!
//! It walks the value with `Peek` and builds the JSON `Value` a family's writer would build
//! member by member, by what the Shape says of each type:
//!
//! - a struct: its fields in declaration order under their JSON names (`rename`,
//!   `rename_all`); a `skip` field, one that `skip_serializing_if` leaves out and an `Option`
//!   that is `None` are not written; a `flatten`ed struct writes its members in the place of
//!   the field, a `flatten`ed internally tagged enum its tag and its variant's members there;
//!   the members of an [`UnknownMembers`] field come after every member the struct wrote, each
//!   whose key no member took (the typed member stands; [`Writing::unknown`] may refuse one
//!   named like a member, or every one); an [`Extra`] marker writes nothing;
//!   a `transparent` struct is its field, a tuple struct an array;
//! - an enum: a unit variant by its JSON name; an internally tagged one (`tag`) as an object
//!   with the tag first; an `untagged` one as its variant's value; any other as facet's
//!   external tagging (`"name"`, or `{"name": value}`);
//! - a list, an array, a set and a tuple as an array; a map as an object, in its order, its
//!   keys written as strings;
//! - a leaf: a container or field `proxy` as its proxy, a `Value` as itself, a Molang value
//!   in the form it was read (`molang::write`; a Molang bool refused where
//!   [`Writing::refuse_molang_bools`] says so), a bool, an integer, a string; a float by the
//!   [`Writing`]'s rule: no JSON spells a float that is not finite, which is refused at its
//!   path.
//!
//! The document's own unknown members (its `Unknown` table) are not the writer's: the envelope
//! writer puts them in by path, after it.

use facet::{Facet, Peek, ScalarType, StructKind, Type, UserType};
use facet::{HasFields, PeekEnum, PeekStruct};

use crate::diagnostic::JsonPath;
use crate::json::{EncodeError, MAX_JSON_DEPTH, Unrepresentable, Value};
use crate::molang::{Molang, MolangList};
use facet_value::{VArray, VObject};

use crate::document::{Extra, UnknownMembers};

/// How the writer spells what the Shape leaves open: the form of a float; and what it refuses
/// that a reader would not read back: an unknown member, a Molang bool.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct Writing {
    /// How an `f32` is written.
    pub floats: Floats,
    /// What becomes of the members of an [`UnknownMembers`] field.
    pub unknown: Unknowns,
    /// Whether a Molang bool is refused at its path: a reader that takes no bool for a Molang
    /// member (particle's 1.20.80 reader) reads none back.
    pub refuse_molang_bools: bool,
}

/// What the writer makes of the members of an [`UnknownMembers`] field.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum Unknowns {
    /// Written after the struct's members, each whose key no member took; the typed member
    /// stands.
    #[default]
    KnownWins,
    /// Written after the struct's members; one named like a member of the struct (written or
    /// not) is refused at its key, since it would replace that member in the text.
    NoneNamedKnown,
    /// Each refused at its key: a document a strict reader reads has no place for one
    /// (particle's 1.20.80 reader).
    Refused,
}

impl Writing {
    /// Floats as reals (`1.0`): what `json::write_json` writes for a float.
    pub const REAL: Self = Self {
        floats: Floats::Real,
        unknown: Unknowns::KnownWins,
        refuse_molang_bools: false,
    };
    /// A whole float as an integer (`1`): the node-tree families (geometry).
    pub const WHOLE: Self = Self {
        floats: Floats::Whole,
        ..Self::REAL
    };

    /// The same writing, with the members of an [`UnknownMembers`] field as `unknown` says.
    #[must_use]
    pub const fn unknown(self, unknown: Unknowns) -> Self {
        Self { unknown, ..self }
    }

    /// The same writing, refusing a Molang bool at its path.
    #[must_use]
    pub const fn refusing_molang_bools(self) -> Self {
        Self {
            refuse_molang_bools: true,
            ..self
        }
    }
}

/// How an `f32` is written; either way the shortest decimal that reads back as the same `f32`
/// (`0.1`, not the widened `0.10000000149011612`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum Floats {
    /// A real: a whole number with its `.0`, so the real tag a reader may look at survives.
    #[default]
    Real,
    /// A whole number below `9e15` in magnitude as an integer, a negative zero as `-0.0`,
    /// the rest as reals.
    Whole,
}

/// `value` as JSON, by its Shape (see the module), floats as reals.
///
/// # Errors
///
/// Returns `EncodeError::NotRepresentable` at the member that has no JSON form: a float that
/// is not finite, a value nested past `MAX_JSON_DEPTH`, a map key that is no string, a type the
/// writer has no form for.
pub fn write_value<'f, T: Facet<'f> + ?Sized>(value: &T) -> Result<Value, EncodeError> {
    write_value_with(value, Writing::REAL)
}

/// [`write_value`] by `writing`'s rules.
///
/// # Errors
///
/// As [`write_value`].
pub fn write_value_with<'f, T: Facet<'f> + ?Sized>(
    value: &T,
    writing: Writing,
) -> Result<Value, EncodeError> {
    let mut walk = Walk {
        writing,
        path: JsonPath::new(),
    };
    walk.value(Peek::new(value))
        .map_err(|what| EncodeError::NotRepresentable {
            at: walk.path,
            what,
        })
}

/// One walk: the rules and the path of the value being written (the path of a refusal).
struct Walk {
    writing: Writing,
    path: JsonPath,
}

type Wrote = Result<Value, Unrepresentable>;

impl Walk {
    /// The value at `peek`, by its shape.
    fn value(&mut self, peek: Peek<'_, '_>) -> Wrote {
        if self.path.len() > MAX_JSON_DEPTH {
            return Err(Unrepresentable::TooDeep);
        }
        crate::json::with_stack_for(1, || self.value_here(peek))
    }

    fn value_here(&mut self, peek: Peek<'_, '_>) -> Wrote {
        let shape = peek.shape();
        if shape.id == Value::SHAPE.id {
            return peek.get::<Value>().cloned().map_err(unsupported);
        }
        if shape.id == Molang::SHAPE.id {
            let value = peek.get::<Molang>().map_err(unsupported)?;
            if self.writing.refuse_molang_bools && matches!(value, Molang::Bool(_)) {
                return Err(Unrepresentable::Unsupported);
            }
            return crate::molang::write(value);
        }
        if shape.id == MolangList::SHAPE.id {
            let list = peek.get::<MolangList>().map_err(unsupported)?;
            if self.writing.refuse_molang_bools {
                self.no_molang_bool(list)?;
            }
            return crate::molang::write_list(list);
        }
        let peek = peek.innermost_peek();
        let shape = peek.shape();
        if let Some(proxy) = peek
            .custom_serialization_from_shape_with_format(Some("json"))
            .map_err(unsupported)?
        {
            return self.value(proxy.as_peek());
        }
        if let Some(scalar) = peek.scalar_type() {
            return self.scalar(scalar, peek);
        }
        if let Ok(option) = peek.into_option() {
            return match option.value() {
                Some(inner) => self.value(inner),
                None => Ok(Value::NULL),
            };
        }
        if let Ok(list) = peek.into_list_like() {
            let mut array = VArray::with_capacity(list.len());
            for (index, element) in list.iter().enumerate() {
                array.push(self.below_index(index, element)?);
            }
            return Ok(array.into());
        }
        if let Ok(set) = peek.into_set() {
            let mut array = VArray::with_capacity(set.len());
            for (index, element) in set.iter().enumerate() {
                array.push(self.below_index(index, element)?);
            }
            return Ok(array.into());
        }
        if let Ok(map) = peek.into_map() {
            let mut object = VObject::with_capacity(map.len());
            for (key, value) in map.iter() {
                let key = self.key(key)?;
                let value = self.below_key(&key, value)?;
                object.insert(key, value);
            }
            return Ok(object.into());
        }
        match shape.ty {
            Type::User(UserType::Struct(_)) => {
                let item = peek.into_struct().map_err(unsupported)?;
                self.structure(shape.is_transparent(), item)
            }
            Type::User(UserType::Enum(_)) => self.enumeration(peek),
            _ => Err(Unrepresentable::Unsupported),
        }
    }

    /// A scalar: a bool, an integer, a float by the writing's rule, a text.
    fn scalar(&self, scalar: ScalarType, peek: Peek<'_, '_>) -> Wrote {
        macro_rules! int {
            ($t:ty) => {
                peek.get::<$t>()
                    .map(|n| Value::from(*n))
                    .map_err(unsupported)
            };
        }
        match scalar {
            ScalarType::Unit => Ok(Value::NULL),
            ScalarType::Bool => int!(bool),
            ScalarType::Char => peek
                .get::<char>()
                .map(|c| Value::from(c.to_string().as_str()))
                .map_err(unsupported),
            ScalarType::U8 => int!(u8),
            ScalarType::U16 => int!(u16),
            ScalarType::U32 => int!(u32),
            ScalarType::U64 => int!(u64),
            ScalarType::USize => int!(usize),
            ScalarType::I8 => int!(i8),
            ScalarType::I16 => int!(i16),
            ScalarType::I32 => int!(i32),
            ScalarType::I64 => int!(i64),
            ScalarType::ISize => int!(isize),
            ScalarType::F32 => {
                let x = *peek.get::<f32>().map_err(unsupported)?;
                self.f32(x)
            }
            ScalarType::F64 => {
                let x = *peek.get::<f64>().map_err(unsupported)?;
                finite(x).map(Value::from)
            }
            _ => peek
                .as_str()
                .map(Value::from)
                .ok_or(Unrepresentable::Unsupported),
        }
    }

    /// An `f32` by the writing's rule.
    #[allow(
        clippy::cast_possible_truncation,
        reason = "a whole number below 9e15 is its own integer"
    )]
    fn f32(&self, x: f32) -> Wrote {
        let x = finite(f64::from(x)).map(|_| x)?;
        if self.writing.floats == Floats::Whole {
            if x == 0.0 && x.is_sign_negative() {
                // The zero keeps its sign: an integer has none.
                return Ok(Value::from(-0.0_f64));
            }
            if x.fract() == 0.0 && x.abs() < 9.0e15 {
                return Ok(Value::from(f64::from(x) as i64));
            }
        }
        let shortest = format!("{x:?}").parse::<f64>().unwrap_or(f64::from(x));
        Ok(Value::from(shortest))
    }

    /// A map key as the string it is written as.
    fn key(&mut self, key: Peek<'_, '_>) -> Result<String, Unrepresentable> {
        if let Some(text) = key.as_str() {
            return Ok(text.to_owned());
        }
        let written = self.value(key)?;
        written
            .as_string()
            .map(|text| text.as_str().to_owned())
            .ok_or(Unrepresentable::Unsupported)
    }

    /// The value of the member `key`, with the path through it.
    fn below_key(&mut self, key: &str, value: Peek<'_, '_>) -> Wrote {
        self.path.push(key);
        let written = self.value(value)?;
        self.path.pop();
        Ok(written)
    }

    /// The element `index`, with the path through it.
    fn below_index(&mut self, index: usize, value: Peek<'_, '_>) -> Wrote {
        self.path.push_index(index);
        let written = self.value(value)?;
        self.path.pop();
        Ok(written)
    }

    /// A struct: a transparent one is its field, a tuple struct an array, a named one an object.
    fn structure(&mut self, transparent: bool, item: PeekStruct<'_, '_>) -> Wrote {
        match item.ty().kind {
            StructKind::Unit => Ok(Value::NULL),
            StructKind::Tuple | StructKind::TupleStruct => {
                let fields: Vec<_> = item.fields().collect();
                if let ([(field, value)], true) = (fields.as_slice(), transparent) {
                    return self.field_value(field, *value);
                }
                let mut array = VArray::with_capacity(fields.len());
                for (index, (field, value)) in fields.into_iter().enumerate() {
                    self.path.push_index(index);
                    array.push(self.field_value(&field, value)?);
                    self.path.pop();
                }
                Ok(array.into())
            }
            StructKind::Struct => {
                let mut object = Object::default();
                self.members(&item, &mut object)?;
                let fields = item.ty().fields;
                self.object(object, |key| {
                    fields
                        .iter()
                        .any(|field| !field.is_flattened() && field.effective_name() == key)
                })
            }
        }
    }

    /// A Molang list without a bool, or the refusal at the first one's path.
    fn no_molang_bool(&mut self, list: &MolangList) -> Result<(), Unrepresentable> {
        match list {
            MolangList::One(Molang::Bool(_)) => Err(Unrepresentable::Unsupported),
            MolangList::One(_) => Ok(()),
            MolangList::Many(elements) => {
                for (index, element) in elements.iter().enumerate() {
                    self.path.push_index(index);
                    self.no_molang_bool(element)?;
                    self.path.pop();
                }
                Ok(())
            }
        }
    }

    /// The object of `object`'s members and then its unknown members, as the writing says;
    /// `known` names the members of the struct written (an unknown member named like one is
    /// refused under [`Unknowns::NoneNamedKnown`]).
    fn object(&mut self, object: Object, known: impl Fn(&str) -> bool) -> Wrote {
        let Object {
            mut members,
            unknown,
        } = object;
        for (key, value) in unknown {
            let refused = match self.writing.unknown {
                Unknowns::KnownWins => false,
                Unknowns::NoneNamedKnown => known(&key) || members.contains_key(&key),
                Unknowns::Refused => true,
            };
            if refused {
                self.path.push(&key);
                return Err(Unrepresentable::Unsupported);
            }
            if !members.contains_key(&key) {
                members.insert(key, value);
            }
        }
        Ok(members.into())
    }

    /// The members of `item` into `object`.
    fn members<'mem, 'facet>(
        &mut self,
        item: &impl HasFields<'mem, 'facet>,
        object: &mut Object,
    ) -> Result<(), Unrepresentable> {
        for (field, value) in item.fields() {
            if skipped(item, &field, value) {
                continue;
            }
            let shape = value.shape();
            if shape.id == Extra::SHAPE.id {
                continue;
            }
            if shape.id == UnknownMembers::SHAPE.id {
                let kept = value.get::<UnknownMembers>().map_err(unsupported)?;
                object
                    .unknown
                    .extend(kept.iter().map(|(k, v)| (k.to_owned(), v.clone())));
                continue;
            }
            let value = match value.into_option() {
                Ok(option) => match option.value() {
                    Some(inner) if field.is_flattened() => inner,
                    Some(_) => value,
                    None => continue,
                },
                Err(_) => value,
            };
            if field.is_flattened() {
                self.flattened(value, object)?;
                continue;
            }
            let name = field.effective_name();
            self.path.push(name);
            let written = self.field_value(&field, value)?;
            self.path.pop();
            object.members.insert(name, written);
        }
        Ok(())
    }

    /// A flattened field's members into `object`: a struct's, or an internally tagged enum's
    /// tag and its variant's.
    fn flattened(
        &mut self,
        value: Peek<'_, '_>,
        object: &mut Object,
    ) -> Result<(), Unrepresentable> {
        let value = value.innermost_peek();
        if let Ok(item) = value.into_struct() {
            return self.members(&item, object);
        }
        let shape = value.shape();
        let Some(tag) = shape.get_tag_attr() else {
            return Err(Unrepresentable::Unsupported);
        };
        let item = value.into_enum().map_err(unsupported)?;
        self.tagged(tag, &item, object)
    }

    /// The tag of an internally tagged enum's variant, then the variant's members.
    fn tagged(
        &mut self,
        tag: &'static str,
        item: &PeekEnum<'_, '_>,
        object: &mut Object,
    ) -> Result<(), Unrepresentable> {
        let variant = item.active_variant().map_err(unsupported)?;
        object
            .members
            .insert(tag, Value::from(variant.effective_name()));
        match variant.data.kind {
            StructKind::Unit => Ok(()),
            StructKind::Struct => self.members(item, object),
            StructKind::Tuple | StructKind::TupleStruct => {
                // A newtype variant: its struct's members beside the tag.
                let inner = item
                    .field(0)
                    .ok()
                    .flatten()
                    .ok_or(Unrepresentable::Unsupported)?;
                self.flattened(inner, object)
            }
        }
    }

    /// An enum by facet's tagging.
    fn enumeration(&mut self, peek: Peek<'_, '_>) -> Wrote {
        let shape = peek.shape();
        let item = peek.into_enum().map_err(unsupported)?;
        let variant = item.active_variant().map_err(unsupported)?;
        let name = variant.effective_name();
        if let Some(tag) = shape.get_tag_attr() {
            let mut object = Object::default();
            self.tagged(tag, &item, &mut object)?;
            return self.object(object, |_| false);
        }
        let payload = match variant.data.kind {
            StructKind::Unit => return Ok(Value::from(name)),
            StructKind::Struct => {
                let mut object = Object::default();
                self.members(&item, &mut object)?;
                self.object(object, |_| false)?
            }
            StructKind::Tuple | StructKind::TupleStruct => {
                let fields: Vec<_> = item.fields().collect();
                if let [(field, value)] = fields.as_slice() {
                    self.field_value(field, *value)?
                } else {
                    let mut array = VArray::with_capacity(fields.len());
                    for (index, (field, value)) in fields.into_iter().enumerate() {
                        self.path.push_index(index);
                        array.push(self.field_value(&field, value)?);
                        self.path.pop();
                    }
                    array.into()
                }
            }
        };
        if shape.is_untagged() {
            return Ok(payload);
        }
        let mut object = VObject::with_capacity(1);
        object.insert(name, payload);
        Ok(object.into())
    }

    /// A field's value, through the field's proxy when it has one.
    fn field_value(&mut self, field: &facet::Field, value: Peek<'_, '_>) -> Wrote {
        if field.effective_proxy(Some("json")).is_some() {
            let proxy = value.custom_serialization(*field).map_err(unsupported)?;
            return self.value(proxy.as_peek());
        }
        self.value(value)
    }
}

/// An object being written: the members, then the kept unknown members whose key no member
/// took.
#[derive(Default)]
struct Object {
    members: VObject,
    unknown: Vec<(String, Value)>,
}

/// Whether `field` (holding `value`) of `owner` is not written: `skip`, `skip_serializing`, or
/// left out by its `skip_serializing_if`, which facet evaluates in its own walk of the owner's
/// fields: the field is skipped when that walk does not yield this value.
fn skipped<'mem, 'facet>(
    owner: &impl HasFields<'mem, 'facet>,
    field: &facet::Field,
    value: Peek<'mem, 'facet>,
) -> bool {
    if field.should_skip_serializing_unconditional() {
        return true;
    }
    if field.skip_serializing_if.is_none() {
        return false;
    }
    let at = value.data().as_byte_ptr();
    !owner.fields_for_serialize().any(|(item, yielded)| {
        !item.flattened
            && yielded.shape().id == value.shape().id
            && std::ptr::eq(yielded.data().as_byte_ptr(), at)
    })
}

/// `x`, or the refusal of a number JSON does not spell.
fn finite(x: f64) -> Result<f64, Unrepresentable> {
    if x.is_nan() {
        Err(Unrepresentable::NaN)
    } else if x == f64::INFINITY {
        Err(Unrepresentable::Infinity)
    } else if x == f64::NEG_INFINITY {
        Err(Unrepresentable::NegativeInfinity)
    } else {
        Ok(x)
    }
}

/// A reflection failure: the value has no form the writer knows.
fn unsupported<E>(_: E) -> Unrepresentable {
    Unrepresentable::Unsupported
}

#[cfg(test)]
mod tests;
