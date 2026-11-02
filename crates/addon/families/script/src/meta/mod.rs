mod generated;
/// The published script `metadata/` files as written (`model::script`, `model::engine`,
/// `model::vanilla`, `model::commands`, `model::molang`).
pub mod model;
pub use descriptor::{Errorability, Prim, TypeDescriptor, TypeKind, TypeOverlay};
pub use error::MetaError;
pub use module::{ModuleMetadata, NativeModule, StubModule, StubOverlay};
pub use privilege::{Privilege, PrivilegeList};
pub use versions::{
    Channel, ModuleEntry, ModuleShape, ModuleVersion, PeerReq, engine_only, module, module_by_uuid,
    modules, samples_tag,
};

// Script-module metadata: the typed model of the published `metadata/` files, and the
// module catalog generated from it. The module's rustdoc is on its declaration in
// `lib.rs`.

mod descriptor;
mod error;
mod module;
mod parse;
mod privilege;
mod versions;
