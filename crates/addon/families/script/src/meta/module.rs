//! `ModuleMetadata`: one script-module file, told apart as a native module or a stub
//!.

use uuid::Uuid;

use super::MetaError;
use super::model::script::{
    Class, Dependency, Enum, ErrorClass, Function, Interface, ModuleConstant, Object, ParentModule,
    PeerDependency, RawModuleFile, TypeAlias,
};
use bedrock_addon_core::json::Value;

/// One script-module file (`metadata/script_modules/@minecraft/<module>_<version>.json`): a
/// native module, which declares the API, or a stub, which a pack names in its manifest and
/// which stands for the bindings module of the same version.
#[derive(Clone, Debug, PartialEq)]
#[allow(clippy::large_enum_variant)] // read once per file
pub enum ModuleMetadata {
    /// A file without `parentModule`.
    Native(NativeModule),
    /// A file with `parentModule`.
    Stub(StubModule),
}

/// A native module: every declaration array is present, possibly empty.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct NativeModule {
    /// `name`: `@minecraft/<module>`.
    pub name: String,
    /// `uuid`.
    pub uuid: Uuid,
    /// `peer_dependencies`.
    pub peer_dependencies: Vec<PeerDependency>,
    /// `dependencies`: empty in every release of the metadata.
    pub dependencies: Vec<Dependency>,
    /// `classes`.
    pub classes: Vec<Class>,
    /// `interfaces`.
    pub interfaces: Vec<Interface>,
    /// `enums`.
    pub enums: Vec<Enum>,
    /// `errors`.
    pub errors: Vec<ErrorClass>,
    /// `functions`.
    pub functions: Vec<Function>,
    /// `objects`.
    pub objects: Vec<Object>,
    /// `constants`.
    pub constants: Vec<ModuleConstant>,
    /// `type_aliases`.
    pub type_aliases: Vec<TypeAlias>,
}

/// A stub module: the public name packs depend on (`@minecraft/server`), its bindings module
/// and the JavaScript wrapper that re-exports it.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct StubModule {
    /// `name`: `@minecraft/<module>`.
    pub name: String,
    /// `uuid`.
    pub uuid: Uuid,
    /// `peer_dependencies`.
    pub peer_dependencies: Vec<PeerDependency>,
    /// `parentModule`: the bindings module of the same version.
    pub parent_module: ParentModule,
    /// `wrapper_module`: the wrapper (`@minecraft/server-bindings-v2-wrapper`).
    pub wrapper_module: String,
    /// Classes, interfaces or enums found on the stub: script-generated overlay, seen in one
    /// file of one older release of the metadata only.
    pub overlay: Option<StubOverlay>,
}

/// Declarations on a stub (`classes`, `interfaces`, `enums`), all script-generated.
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct StubOverlay {
    /// `classes`.
    pub classes: Vec<Class>,
    /// `interfaces`.
    pub interfaces: Vec<Interface>,
    /// `enums`.
    pub enums: Vec<Enum>,
}

impl ModuleMetadata {
    /// Reads a script-module file strictly and classifies it
    /// ([`RawModuleFile::classify`]); `details_key_present` of every argument is filled from
    /// the text.
    pub fn parse(bytes: &[u8]) -> Result<Self, MetaError> {
        let (mut file, json): (RawModuleFile, Value) = super::parse::strict(bytes)?;
        mark_details(&mut file, &json);
        file.classify()
    }

    /// `name`.
    pub fn name(&self) -> &str {
        match self {
            Self::Native(native) => &native.name,
            Self::Stub(stub) => &stub.name,
        }
    }

    /// `uuid`.
    pub fn uuid(&self) -> Uuid {
        match self {
            Self::Native(native) => native.uuid,
            Self::Stub(stub) => stub.uuid,
        }
    }

    /// `peer_dependencies`.
    pub fn peer_dependencies(&self) -> &[PeerDependency] {
        match self {
            Self::Native(native) => &native.peer_dependencies,
            Self::Stub(stub) => &stub.peer_dependencies,
        }
    }
}

impl RawModuleFile {
    /// Tells the file apart: a stub iff `parentModule` is present, which needs
    /// `wrapper_module` and none of `dependencies`, `constants`, `errors`, `functions`,
    /// `objects`, `type_aliases` (`classes`, `interfaces`, `enums` become its overlay); a
    /// native module otherwise, which needs all nine declaration arrays and no
    /// `wrapper_module`.
    pub fn classify(self) -> Result<ModuleMetadata, MetaError> {
        let uuid = Uuid::parse_str(&self.uuid)?;
        let Some(parent_module) = self.parent_module else {
            if self.wrapper_module.is_some() {
                return Err(MetaError::WrapperWithoutParent);
            }
            let need = |member| MetaError::MissingArray { member };
            return Ok(ModuleMetadata::Native(NativeModule {
                name: self.name,
                uuid,
                peer_dependencies: self.peer_dependencies,
                dependencies: self.dependencies.ok_or_else(|| need("dependencies"))?,
                classes: self.classes.ok_or_else(|| need("classes"))?,
                interfaces: self.interfaces.ok_or_else(|| need("interfaces"))?,
                enums: self.enums.ok_or_else(|| need("enums"))?,
                errors: self.errors.ok_or_else(|| need("errors"))?,
                functions: self.functions.ok_or_else(|| need("functions"))?,
                objects: self.objects.ok_or_else(|| need("objects"))?,
                constants: self.constants.ok_or_else(|| need("constants"))?,
                type_aliases: self.type_aliases.ok_or_else(|| need("type_aliases"))?,
            }));
        };
        let wrapper_module = self.wrapper_module.ok_or(MetaError::StubWithoutWrapper)?;
        let native_only = [
            ("dependencies", self.dependencies.is_some()),
            ("constants", self.constants.is_some()),
            ("errors", self.errors.is_some()),
            ("functions", self.functions.is_some()),
            ("objects", self.objects.is_some()),
            ("type_aliases", self.type_aliases.is_some()),
        ];
        if let Some((member, _)) = native_only.into_iter().find(|(_, present)| *present) {
            return Err(MetaError::StubMember { member });
        }
        let overlay = (self.classes.is_some() || self.interfaces.is_some() || self.enums.is_some())
            .then(|| StubOverlay {
                classes: self.classes.unwrap_or_default(),
                interfaces: self.interfaces.unwrap_or_default(),
                enums: self.enums.unwrap_or_default(),
            });
        Ok(ModuleMetadata::Stub(StubModule {
            name: self.name,
            uuid,
            peer_dependencies: self.peer_dependencies,
            parent_module,
            wrapper_module,
            overlay,
        }))
    }
}

/// Sets `details_key_present` on every argument of every function of `file` from the same
/// text read as `json`: `null` and an absent key both give `details: None`.
fn mark_details(file: &mut RawModuleFile, json: &Value) {
    fn member<'v>(value: &'v Value, key: &str) -> Option<&'v Value> {
        value.as_object().and_then(|object| object.get(key))
    }
    fn elements(value: Option<&Value>) -> Vec<&Value> {
        value
            .and_then(Value::as_array)
            .map(|array| array.iter().collect())
            .unwrap_or_default()
    }
    if let Some(functions) = &mut file.functions {
        mark_functions(functions, member(json, "functions"));
    }
    for (class, json) in file
        .classes
        .iter_mut()
        .flatten()
        .zip(elements(member(json, "classes")))
    {
        mark_functions(&mut class.functions, member(json, "functions"));
    }
    for (error, json) in file
        .errors
        .iter_mut()
        .flatten()
        .zip(elements(member(json, "errors")))
    {
        mark_functions(&mut error.functions, member(json, "functions"));
    }
    for (interface, json) in file
        .interfaces
        .iter_mut()
        .flatten()
        .zip(elements(member(json, "interfaces")))
    {
        if let Some(functions) = &mut interface.functions {
            mark_functions(functions, member(json, "functions"));
        }
    }
}

/// [`mark_details`] for one `functions` array and its JSON.
fn mark_functions(functions: &mut [Function], json: Option<&Value>) {
    let Some(array) = json.and_then(Value::as_array) else {
        return;
    };
    for (function, json) in functions.iter_mut().zip(array.iter()) {
        let Some(arguments) = json
            .as_object()
            .and_then(|object| object.get("arguments"))
            .and_then(Value::as_array)
        else {
            continue;
        };
        for (argument, json) in function.arguments.iter_mut().zip(arguments.iter()) {
            argument.details_key_present = json
                .as_object()
                .is_some_and(|object| object.contains_key("details"));
        }
    }
}
