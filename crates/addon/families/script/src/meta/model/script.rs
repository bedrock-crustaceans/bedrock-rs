//! The script-module files, `metadata/script_modules/@minecraft/<module>_<version>.json`: one
//! struct per JSON object kind, every key the game's metadata generator writes.
//!
//! A key present in every object of its kind in every release of the metadata is a plain field; a
//! key that is sometimes absent is an `Option`. The files are machine-written, so a key no field
//! takes is an error ([`MetaError::UnknownMember`](crate::meta::MetaError::UnknownMember))
//! rather than kept.

use facet::Facet;
use facet_value::Value;

use super::ShapeError;
use bedrock_addon_core::json::ScalarShape;

/// A script-module file as written, before it is told apart as a native module or a stub
/// ([`RawModuleFile::classify`]); [`ModuleMetadata::parse`](crate::meta::ModuleMetadata::parse)
/// reads and classifies in one step. The file carries no version: a module's version is only in
/// its file name.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct RawModuleFile {
    /// `name`: `@minecraft/<module>`, the file name's prefix.
    pub name: String,
    /// `uuid`: the module's UUID, in either case (`@minecraft/server-ui`'s is upper case).
    pub uuid: String,
    /// `module_type`: always `script`.
    pub module_type: ModuleType,
    /// `peer_dependencies`: in every file, possibly empty.
    pub peer_dependencies: Vec<PeerDependency>,
    /// `parentModule`: the bindings module a stub stands for; stubs only.
    #[facet(rename = "parentModule")]
    pub parent_module: Option<ParentModule>,
    /// `wrapper_module`: the JavaScript wrapper that re-exports a stub's parent
    /// (`@minecraft/server-bindings-v2-wrapper`); stubs only.
    pub wrapper_module: Option<String>,
    /// `dependencies`: native modules only; empty in every release of the metadata.
    pub dependencies: Option<Vec<Dependency>>,
    /// `classes`: native modules; on a stub only as script-generated overlay.
    pub classes: Option<Vec<Class>>,
    /// `interfaces`: native modules; on a stub only as script-generated overlay.
    pub interfaces: Option<Vec<Interface>>,
    /// `enums`: native modules; on a stub only as script-generated overlay.
    pub enums: Option<Vec<Enum>>,
    /// `errors`: native modules only.
    pub errors: Option<Vec<ErrorClass>>,
    /// `functions`: the module's free functions; native modules only.
    pub functions: Option<Vec<Function>>,
    /// `objects`: the module's global objects (`world`, `system`); native modules only.
    pub objects: Option<Vec<Object>>,
    /// `constants`: the module's constants; native modules only.
    pub constants: Option<Vec<ModuleConstant>>,
    /// `type_aliases`: native modules only.
    pub type_aliases: Option<Vec<TypeAlias>>,
}

/// `module_type` of a script-module file.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet)]
#[repr(u8)]
pub enum ModuleType {
    /// `script`.
    #[facet(rename = "script")]
    Script,
}

/// `parentModule` of a stub: the bindings module that declares what the stub exports.
#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct ParentModule {
    /// `name`: the bindings module (`@minecraft/server-bindings`).
    pub name: String,
    /// `version`: the bindings version; always the stub's own version.
    pub version: String,
}

/// One entry of `dependencies`. No release of the metadata has one; it is read like a peer so a first real
/// entry parses.
#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct Dependency {
    /// `name`.
    pub name: String,
    /// `uuid`.
    pub uuid: Option<String>,
    /// `version`.
    pub version: Option<String>,
}

/// One entry of `peer_dependencies`: a public module this one works with, and the versions it
/// accepts.
#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct PeerDependency {
    /// `name`: the public module (`@minecraft/server`, `@minecraft/common`,
    /// `@minecraft/vanilla-data`).
    pub name: String,
    /// `uuid`: absent on `@minecraft/vanilla-data` peers and on stub peers.
    pub uuid: Option<String>,
    /// `versions`: the accepted versions, one per major plus the current beta.
    pub versions: Vec<VersionRef>,
    /// `version`: only beside `types_only` (the npm `@minecraft/vanilla-data` pin `1.20.70`).
    pub version: Option<String>,
    /// `types_only`: `true` on the `@minecraft/vanilla-data` peer, an npm type package; never
    /// `false` when present.
    pub types_only: Option<bool>,
    /// `wrapperName`: only on `@minecraft/server-ui` 2.x stub peers
    /// (`@minecraft/server-wrapper`).
    #[facet(rename = "wrapperName")]
    pub wrapper_name: Option<String>,
}

/// One entry of `peer_dependencies[].versions`.
#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct VersionRef {
    /// `version`.
    pub version: String,
}

/// One entry of `classes`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Class {
    /// `name`.
    pub name: String,
    /// `type`: the class's own descriptor (`is_bind_type: true`).
    #[facet(rename = "type")]
    pub ty: RawTypeDesc,
    /// `base_types`: the classes it extends; absent on a few script-generated classes.
    #[facet(default)]
    pub base_types: Vec<RawTypeDesc>,
    /// `constants`: possibly empty.
    pub constants: Vec<ClassConstant>,
    /// `functions`: possibly empty.
    pub functions: Vec<Function>,
    /// `properties`: possibly empty.
    pub properties: Vec<Property>,
    /// `iterator`: the descriptor of the value its `next()` returns, on iterable classes.
    pub iterator: Option<RawTypeDesc>,
    /// `runtime_conditions`: conditions the class needs at run time (`["Poi"]`).
    pub runtime_conditions: Option<Vec<String>>,
    /// `generic_class_types`: the type parameters of a script-generated class.
    pub generic_class_types: Option<Vec<GenericParam>>,
    /// `is_script_generated`: the class comes from the TypeScript overlay.
    pub is_script_generated: Option<bool>,
    /// `raw_script_text`: the overlay's TypeScript text.
    pub raw_script_text: Option<String>,
    /// `raw_tsdoc_text`: the overlay's doc comment.
    pub raw_tsdoc_text: Option<String>,
}

/// One entry of `functions`: a class method, an error method or a module function.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Function {
    /// `name`.
    pub name: String,
    /// `arguments`: possibly empty.
    pub arguments: Vec<Argument>,
    /// `return_type`.
    pub return_type: RawTypeDesc,
    /// `call_privilege`: who may call it, a list of names
    /// ([`PrivilegeList::from_names`](crate::meta::PrivilegeList::from_names)).
    pub call_privilege: Vec<PrivilegeName>,
    /// `is_constructor`.
    pub is_constructor: bool,
    /// `is_static`: on every module function, missing on some class functions.
    pub is_static: Option<bool>,
    /// `generic_function_types`: the type parameters of a generic method.
    pub generic_function_types: Option<Vec<GenericParam>>,
    /// `is_script_generated`: the function comes from the TypeScript overlay.
    pub is_script_generated: Option<bool>,
    /// `raw_script_text`: the overlay's TypeScript signature.
    pub raw_script_text: Option<String>,
    /// `raw_tsdoc_text`: the overlay's doc comment.
    pub raw_tsdoc_text: Option<String>,
}

/// One type parameter (`generic_function_types[]`, `generic_class_types[]`).
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct GenericParam {
    /// `name`: the parameter (`T`).
    pub name: String,
    /// `constraint`: what it extends.
    pub constraint: Option<RawTypeDesc>,
    /// `default`: its default type.
    pub default: Option<RawTypeDesc>,
}

/// One entry of `arguments`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Argument {
    /// `name`.
    pub name: String,
    /// `type`.
    #[facet(rename = "type")]
    pub ty: RawTypeDesc,
    /// `details`: bounds and default; `None` both for `null` and for an absent key
    /// ([`details_key_present`](Self::details_key_present) tells them apart).
    pub details: Option<ArgDetails>,
    /// Whether the file has a `details` key: `null` is written for an argument without details,
    /// and script-generated arguments have no key at all. Not a JSON member; filled by
    /// [`ModuleMetadata::parse`](crate::meta::ModuleMetadata::parse).
    #[facet(skip, rename = "\u{0}details_key_present")]
    pub details_key_present: bool,
}

/// `arguments[].details`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct ArgDetails {
    /// `default_value`: a string (`"null"`, `"Simulated Player"`), an integer, a boolean or `[]`,
    /// kept as written.
    pub default_value: Option<Value>,
    /// `min_value`: an integer or, once, a float.
    pub min_value: Option<f64>,
    /// `max_value`: an integer.
    pub max_value: Option<f64>,
    /// `max_length`: the longest string accepted.
    pub max_length: Option<u32>,
}

/// One entry of `properties` of a class, an interface or an error.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Property {
    /// `name`.
    pub name: String,
    /// `type`.
    #[facet(rename = "type")]
    pub ty: RawTypeDesc,
    /// `get_privilege`: who may read it; on every class and error property, never on an
    /// interface property.
    pub get_privilege: Option<Vec<PrivilegeName>>,
    /// `set_privilege`: who may write it; writable class properties only.
    pub set_privilege: Option<Vec<PrivilegeName>>,
    /// `is_read_only`.
    pub is_read_only: bool,
    /// `is_baked`: missing on some script-generated interface properties.
    pub is_baked: Option<bool>,
    /// `min_value`: the smallest value accepted.
    pub min_value: Option<i64>,
    /// `max_value`: the largest value accepted.
    pub max_value: Option<i64>,
    /// `is_script_generated`: the property comes from the TypeScript overlay.
    pub is_script_generated: Option<bool>,
    /// `raw_script_text`: the overlay's TypeScript text.
    pub raw_script_text: Option<String>,
    /// `raw_tsdoc_text`: the overlay's doc comment.
    pub raw_tsdoc_text: Option<String>,
}

/// One entry of `constants` of a class or of `enums[].constants`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct ClassConstant {
    /// `name`.
    pub name: String,
    /// `type`.
    #[facet(rename = "type")]
    pub ty: RawTypeDesc,
    /// `value`: a string or an integer.
    pub value: ConstValue,
    /// `is_read_only`: always `true`.
    pub is_read_only: bool,
    /// `is_static`: always `true`.
    pub is_static: bool,
    /// `is_script_generated`: the constant comes from the TypeScript overlay.
    pub is_script_generated: Option<bool>,
    /// `raw_script_text`: the overlay's TypeScript text.
    pub raw_script_text: Option<String>,
    /// `raw_tsdoc_text`: the overlay's doc comment.
    pub raw_tsdoc_text: Option<String>,
}

/// One entry of the module's `constants`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct ModuleConstant {
    /// `name`.
    pub name: String,
    /// `type`.
    #[facet(rename = "type")]
    pub ty: RawTypeDesc,
    /// `is_read_only`: always `true`.
    pub is_read_only: bool,
    /// `is_static`: always `true`.
    pub is_static: bool,
    /// `value`: an integer in every release of the metadata; a string would parse too.
    pub value: ConstValue,
}

/// The value of a constant: a JSON string or a JSON integer, never anything else. An enum's
/// members are all of one kind; integers occur where the type is an integer primitive or a
/// numeric enum (`PaletteColor`, `HudElement`).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Facet)]
#[facet(proxy = ScalarShape)]
#[repr(u8)]
pub enum ConstValue {
    /// A JSON string.
    Str(String),
    /// A JSON integer.
    Int(i64),
}

/// One entry of `enums`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Enum {
    /// `name`.
    pub name: String,
    /// `constants`: the members.
    pub constants: Vec<ClassConstant>,
    /// `is_script_generated`: the enum comes from the TypeScript overlay.
    pub is_script_generated: Option<bool>,
    /// `raw_script_text`: the overlay's TypeScript text.
    pub raw_script_text: Option<String>,
    /// `raw_tsdoc_text`: the overlay's doc comment.
    pub raw_tsdoc_text: Option<String>,
}

/// One entry of `interfaces`: a plain object shape.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Interface {
    /// `name`.
    pub name: String,
    /// `type`: the interface's own descriptor.
    #[facet(rename = "type")]
    pub ty: RawTypeDesc,
    /// `base_types`: the interfaces it extends.
    pub base_types: Vec<RawTypeDesc>,
    /// `properties`: possibly empty; they carry no privileges.
    pub properties: Vec<Property>,
    /// `constants`: only on script-generated interfaces, always empty.
    pub constants: Option<Vec<ClassConstant>>,
    /// `functions`: only on script-generated interfaces, always empty.
    pub functions: Option<Vec<Function>>,
    /// `is_script_generated`: the interface comes from the TypeScript overlay.
    pub is_script_generated: Option<bool>,
    /// `raw_script_text`: the overlay's TypeScript text.
    pub raw_script_text: Option<String>,
    /// `raw_tsdoc_text`: the overlay's doc comment.
    pub raw_tsdoc_text: Option<String>,
}

/// One entry of `errors`: an error class scripts can catch.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct ErrorClass {
    /// `name`.
    pub name: String,
    /// `type`: the error's own descriptor.
    #[facet(rename = "type")]
    pub ty: RawTypeDesc,
    /// `properties`: possibly empty.
    pub properties: Vec<Property>,
    /// `functions`: possibly empty.
    pub functions: Vec<Function>,
}

/// One entry of `objects`: a global object the module exports (`world`, `system`).
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Object {
    /// `name`.
    pub name: String,
    /// `type`.
    #[facet(rename = "type")]
    pub ty: RawTypeDesc,
    /// `is_read_only`.
    pub is_read_only: bool,
    /// `is_static`.
    pub is_static: bool,
}

/// One entry of `type_aliases`: either a type map (`alias_type` + `mappings`) or a
/// script-generated TypeScript alias (`is_script_generated` + `raw_script_text`).
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct TypeAlias {
    /// `name`.
    pub name: String,
    /// `type`.
    #[facet(rename = "type")]
    pub ty: RawTypeDesc,
    /// `alias_type`: the type-map form.
    pub alias_type: Option<AliasType>,
    /// `mappings`: the type map's entries.
    pub mappings: Option<Vec<AliasMapping>>,
    /// `is_script_generated`: the TypeScript form.
    pub is_script_generated: Option<bool>,
    /// `raw_script_text`: the TypeScript text.
    pub raw_script_text: Option<String>,
}

/// `alias_type` of a type alias.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet)]
#[repr(u8)]
pub enum AliasType {
    /// `type_map`.
    #[facet(rename = "type_map")]
    TypeMap,
}

/// One entry of a type map (`mappings[]`): a component id and the class it returns.
#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct AliasMapping {
    /// `name`: the key (`minecraft:inventory`).
    pub name: String,
    /// `value`: the class (`BlockInventoryComponent`).
    pub value: String,
}

/// `from_module` of a descriptor: the native module that declares a foreign type. It is
/// informational only and does not follow the peer dependencies.
#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct FromModule {
    /// `name`: the declaring module (`@minecraft/server-bindings`).
    pub name: String,
    /// `uuid`: its UUID.
    pub uuid: String,
    /// `version`: its version.
    pub version: String,
}

/// `valid_range` of a numeric descriptor: always the `int32` range, whatever the type, so it
/// says nothing; the real bounds are in argument details and property `min_value` / `max_value`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Facet)]
pub struct ValidRange {
    /// `min`.
    pub min: i64,
    /// `max`.
    pub max: i64,
}

/// One privilege name (`{ "name": … }`): `default`, `restricted_execution`, `early_execution`,
/// or a script-generated name such as `read_only`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Facet)]
pub struct PrivilegeName {
    /// `name`.
    pub name: String,
}

/// A type descriptor as written: every `type`, `return_type`, base type, error type and nested
/// type. [`TypeDescriptor::lower`](crate::meta::TypeDescriptor::lower) gives the form
/// consumers use.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct RawTypeDesc {
    /// `name`: a primitive, a composite tag (`optional`, `array`, …), a declared type, or
    /// TypeScript text.
    pub name: String,
    /// `is_bind_type`: a declared class, interface, enum, error or alias.
    pub is_bind_type: bool,
    /// `is_errorable`: the call may throw.
    pub is_errorable: bool,
    /// `error_types`: what it throws; only with `is_errorable`.
    pub error_types: Option<Vec<RawTypeDesc>>,
    /// `from_module`: the declaring native module of a foreign type.
    pub from_module: Option<FromModule>,
    /// `valid_range`: on numeric primitives; meaningless.
    pub valid_range: Option<ValidRange>,
    /// `optional_type`: the payload of `optional`.
    pub optional_type: Option<Box<RawTypeDesc>>,
    /// `element_type`: the payload of `array`.
    pub element_type: Option<Box<RawTypeDesc>>,
    /// `key_type`: the key payload of `map`.
    pub key_type: Option<Box<RawTypeDesc>>,
    /// `value_type`: the value payload of `map`.
    pub value_type: Option<Box<RawTypeDesc>>,
    /// `variant_types`: the payload of `variant`.
    pub variant_types: Option<Vec<RawTypeDesc>>,
    /// `promise_type`: the payload of `promise`.
    pub promise_type: Option<Box<RawTypeDesc>>,
    /// `closure_type`: the payload of `closure`.
    pub closure_type: Option<Box<ClosureType>>,
    /// `generator_type`: the payload of `generator`.
    pub generator_type: Option<Box<GeneratorType>>,
    /// `generic_base`: the generic type a TypeScript instantiation applies.
    pub generic_base: Option<Box<RawTypeDesc>>,
    /// `generic_types`: the type arguments of a TypeScript instantiation.
    pub generic_types: Option<Vec<RawTypeDesc>>,
    /// `is_core`: a TypeScript overlay flag.
    pub is_core: Option<bool>,
    /// `is_class`: a TypeScript overlay flag.
    pub is_class: Option<bool>,
    /// `is_interface`: a TypeScript overlay flag.
    pub is_interface: Option<bool>,
    /// `is_enum`: a TypeScript overlay flag.
    pub is_enum: Option<bool>,
    /// `is_type`: a TypeScript overlay flag.
    pub is_type: Option<bool>,
    /// `is_optional`: a TypeScript overlay flag.
    pub is_optional: Option<bool>,
}

/// `closure_type`: a callback's signature.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct ClosureType {
    /// `argument_types`: possibly empty.
    pub argument_types: Vec<RawTypeDesc>,
    /// `return_type`.
    pub return_type: RawTypeDesc,
    /// `call_privilege`: the privilege the callback runs with; one object, not a list. Absent
    /// on some script-generated closures.
    pub call_privilege: Option<PrivilegeName>,
}

/// `generator_type`: a generator's three types (`System.runJob`).
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct GeneratorType {
    /// `yield_type`.
    pub yield_type: RawTypeDesc,
    /// `return_type`.
    pub return_type: RawTypeDesc,
    /// `next_type`.
    pub next_type: RawTypeDesc,
}

impl TryFrom<ScalarShape> for ConstValue {
    type Error = ShapeError;

    fn try_from(json: ScalarShape) -> Result<Self, ShapeError> {
        match json {
            ScalarShape::String(text) => Ok(Self::Str(text)),
            ScalarShape::Int(int) => Ok(Self::Int(int)),
            ScalarShape::UInt(uint) => i64::try_from(uint)
                .map(Self::Int)
                .map_err(|_| ShapeError::OutOfRange),
            _ => Err(ShapeError::WrongType {
                expected: "a string or an integer",
            }),
        }
    }
}

impl From<&ConstValue> for ScalarShape {
    fn from(value: &ConstValue) -> Self {
        match value {
            ConstValue::Str(text) => ScalarShape::String(text.clone()),
            ConstValue::Int(int) => match u64::try_from(*int) {
                Ok(uint) => ScalarShape::UInt(uint),
                Err(_) => ScalarShape::Int(*int),
            },
        }
    }
}
