//! [`Vec3`] and [`Vec2`]: a vector member written as an array or as an object of named
//! coordinates, kept in the form it was written (entity's goal members, the colour-grading
//! controls).

use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::json::{Value, ValueExt};
use bedrock_addon_core::molang::{JsonType, MolangFields, MolangVisitor};
use bedrock_addon_core::schema::{SchemaCx, SchemaFields};
use facet_value::{VArray, VObject};

/// Why a vector's JSON could not be read, or a vector could not be written as JSON.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum VectorError {
    /// A value of a JSON type the member does not take.
    #[error("expected {expected}, found a JSON {found}")]
    WrongType {
        /// What the member takes, in words.
        expected: &'static str,
        /// What the document has.
        found: JsonType,
    },
    /// An array of the wrong length.
    #[error("expected an array of {expected} numbers, found {len} elements")]
    ArrayLength {
        /// The length the member takes.
        expected: usize,
        /// The array's length.
        len: usize,
    },
    /// A number with no value of the coordinate type, or a coordinate with no JSON form.
    #[error("the number does not fit the member's type")]
    OutOfRange,
}

/// The JSON form a [`Vec2`] or [`Vec3`] was read from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum VecForm {
    /// `[x, y, z]`.
    #[default]
    Array,
    /// `{"x": …, "y": …, "z": …}`; a missing coordinate is 0.
    Object,
}

/// What a coordinate of a [`Vec3`] or [`Vec2`] is: read from one JSON number, written as one.
pub trait Coordinate: Copy + Default + PartialEq {
    /// The coordinate `value` holds.
    ///
    /// # Errors
    ///
    /// [`VectorError::WrongType`] for a value that is no number, [`VectorError::OutOfRange`]
    /// for a number the type has no value for.
    fn read(value: &Value) -> Result<Self, VectorError>;

    /// The coordinate as a JSON number.
    ///
    /// # Errors
    ///
    /// [`VectorError::OutOfRange`] for a coordinate with no JSON form.
    fn write(self) -> Result<Value, VectorError>;
}

/// Any number, as the nearest `f32`, when that is finite; written as a real in its shortest
/// decimal form.
impl Coordinate for f32 {
    fn read(value: &Value) -> Result<Self, VectorError> {
        let x = value.as_number().ok_or(VectorError::WrongType {
            expected: "a number",
            found: value.json_type(),
        })?;
        #[allow(clippy::cast_possible_truncation)] // the nearest f32 is the reading
        let x = x.to_f64_lossy() as f32;
        if x.is_finite() {
            Ok(x)
        } else {
            Err(VectorError::OutOfRange)
        }
    }

    fn write(self) -> Result<Value, VectorError> {
        if !self.is_finite() {
            return Err(VectorError::OutOfRange);
        }
        let shortest: f64 = format!("{self:?}")
            .parse()
            .map_err(|_| VectorError::OutOfRange)?;
        Ok(Value::from(shortest))
    }
}

/// Reads the coordinates of a vector value: an array of exactly `N` numbers, or an object with
/// the members `names` (each defaulting to 0; other members are returned).
fn read_vector<T: Coordinate, const N: usize>(
    value: &Value,
    names: [&'static str; N],
) -> Result<([T; N], VecForm, UnknownMembers), VectorError> {
    let mut out = [T::default(); N];
    if let Some(array) = value.as_array() {
        if array.len() != N {
            return Err(VectorError::ArrayLength {
                expected: N,
                len: array.len(),
            });
        }
        for (slot, element) in out.iter_mut().zip(array.iter()) {
            *slot = T::read(element)?;
        }
        Ok((out, VecForm::Array, UnknownMembers::default()))
    } else if let Some(object) = value.as_object() {
        let mut extra = UnknownMembers::default();
        for (key, element) in object {
            match names.iter().position(|n| *n == key.as_str()) {
                Some(i) => out[i] = T::read(element)?,
                None => {
                    extra.insert(key.as_str().to_owned(), element.clone());
                }
            }
        }
        Ok((out, VecForm::Object, extra))
    } else {
        Err(VectorError::WrongType {
            expected: "an array or an object",
            found: value.json_type(),
        })
    }
}

/// Writes a vector in `form`, the object form followed by its unknown members.
fn write_vector<T: Coordinate, const N: usize>(
    coordinates: [T; N],
    names: [&'static str; N],
    form: VecForm,
    extra: &UnknownMembers,
) -> Result<Value, VectorError> {
    match form {
        VecForm::Array => {
            let mut array = VArray::new();
            for x in coordinates {
                array.push(x.write()?);
            }
            Ok(Value::from(array))
        }
        VecForm::Object => {
            let mut object = VObject::new();
            for (name, x) in names.into_iter().zip(coordinates) {
                object.insert(name, x.write()?);
            }
            for (key, value) in extra.iter() {
                object.insert(key, value.clone());
            }
            Ok(Value::from(object))
        }
    }
}

/// A 3D vector member (`target_range`, `search_area`, …): `[x, y, z]` or `{x, y, z}`.
#[derive(Clone, Debug, Default, PartialEq, facet::Facet)]
#[facet(opaque, proxy = Value)]
pub struct Vec3<T: Coordinate = f32> {
    /// The x coordinate.
    pub x: T,
    /// The y coordinate.
    pub y: T,
    /// The z coordinate.
    pub z: T,
    /// The form read; writing uses it.
    pub form: VecForm,
    /// Members of the object form the vector does not define, kept for write-back.
    pub extra: UnknownMembers,
}

impl<T: Coordinate> Vec3<T> {
    /// The vector `[x, y, z]`, written as an array.
    pub fn new(x: T, y: T, z: T) -> Self {
        Self {
            x,
            y,
            z,
            form: VecForm::Array,
            extra: UnknownMembers::default(),
        }
    }

    /// The vector `[v, v, v]`, written as an array.
    pub fn splat(v: T) -> Self {
        Self::new(v, v, v)
    }

    /// The coordinates `[x, y, z]`.
    pub fn to_array(&self) -> [T; 3] {
        [self.x, self.y, self.z]
    }
}

impl<T: Coordinate> From<[T; 3]> for Vec3<T> {
    fn from([x, y, z]: [T; 3]) -> Self {
        Self::new(x, y, z)
    }
}

impl<T: Coordinate> TryFrom<Value> for Vec3<T> {
    type Error = VectorError;

    fn try_from(value: Value) -> Result<Self, VectorError> {
        let ([x, y, z], form, extra) = read_vector(&value, ["x", "y", "z"])?;
        Ok(Self {
            x,
            y,
            z,
            form,
            extra,
        })
    }
}

impl<T: Coordinate> TryFrom<&Vec3<T>> for Value {
    type Error = VectorError;

    fn try_from(v: &Vec3<T>) -> Result<Self, VectorError> {
        write_vector([v.x, v.y, v.z], ["x", "y", "z"], v.form, &v.extra)
    }
}

/// A 2D vector member (`transport_items.search_distance`): `[x, y]` or `{x, y}`.
#[derive(Clone, Debug, Default, PartialEq, facet::Facet)]
#[facet(opaque, proxy = Value)]
pub struct Vec2<T: Coordinate = f32> {
    /// The x coordinate.
    pub x: T,
    /// The y coordinate.
    pub y: T,
    /// The form read; writing uses it.
    pub form: VecForm,
    /// Members of the object form the vector does not define, kept for write-back.
    pub extra: UnknownMembers,
}

impl<T: Coordinate> Vec2<T> {
    /// The vector `[x, y]`, written as an array.
    pub fn new(x: T, y: T) -> Self {
        Self {
            x,
            y,
            form: VecForm::Array,
            extra: UnknownMembers::default(),
        }
    }

    /// The coordinates `[x, y]`.
    pub fn to_array(&self) -> [T; 2] {
        [self.x, self.y]
    }
}

impl<T: Coordinate> From<[T; 2]> for Vec2<T> {
    fn from([x, y]: [T; 2]) -> Self {
        Self::new(x, y)
    }
}

impl<T: Coordinate> TryFrom<Value> for Vec2<T> {
    type Error = VectorError;

    fn try_from(value: Value) -> Result<Self, VectorError> {
        let ([x, y], form, extra) = read_vector(&value, ["x", "y"])?;
        Ok(Self { x, y, form, extra })
    }
}

impl<T: Coordinate> TryFrom<&Vec2<T>> for Value {
    type Error = VectorError;

    fn try_from(v: &Vec2<T>) -> Result<Self, VectorError> {
        write_vector([v.x, v.y], ["x", "y"], v.form, &v.extra)
    }
}

// A proxied leaf: no member is walked, nothing in it is Molang.
macro_rules! leaf {
    ($($ty:ident),+) => {
        $(
            impl<T: Coordinate> MolangFields for $ty<T> {
                fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
            }

            impl<T: Coordinate> SchemaFields for $ty<T> {
                fn check_fields(&mut self, _: &mut SchemaCx<'_>) {}

                fn trigger_members(_: &str, _: &mut Vec<String>) {}
            }
        )+
    };
}

leaf!(Vec3, Vec2);
