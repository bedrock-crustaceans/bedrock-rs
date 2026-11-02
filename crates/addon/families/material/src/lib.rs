//! Materials: the list files `materials/common.json`, `fancy.json` and `sad.json`, and the
//! `.material` files they name, loaded together as the `Materials` group of a resource stack.
//! The client still loads these JSON files in full although `RenderDragon` draws with compiled
//! materials: their names are the material namespace of client entities, attachables, render
//! controllers and particles. The server never renders; the family exists to report what a
//! client will reject, with the client's own messages, and to supply the name index.

#![cfg_attr(docsrs, feature(doc_cfg))]

mod file;
mod group;
mod list;
mod load;
mod patch;
mod read;
mod resolve;
mod set;
mod span;
mod values;
mod write;

#[cfg(test)]
mod tests;

pub use file::{
    KeyRules, Listed, MaterialDef, MaterialFile, MaterialFileForm, MaterialKey, SamplerState,
    StencilFace, StencilRefOverride, Variant, VertexField,
};
pub use group::{Material, MaterialGroup, MaterialGroupKind};
pub use list::{ListEntry, MaterialList, MaterialListKind};
pub use resolve::{ResolvedFace, ResolvedMaterial, ResolvedSampler};
pub use values::{
    ACTOR_SHADER_STEMS, BLEND_TARGETS, COMPARISON_FUNCS, IGNORED_RENDER_STATE, MSAA_SUPPORTS,
    PRIMITIVE_MODES, RENDER_STATES, SAMPLER_SLOTS, SHADER_STAGES, STENCIL_OPS, TEXTURE_FILTERS,
    TEXTURE_FORMATS, TEXTURE_WRAPS, VERTEX_FIELDS, render_state_bit,
};

#[doc(hidden)]
pub use load::load_group;

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics};
use bedrock_addon_core::document::{Document, Merged, RawLayer, Rejected};
use bedrock_addon_core::id::MaterialId;
use bedrock_addon_core::pack_identity::PackPath;
use indexmap::IndexMap;

/// The materials of a resource stack (`rp.materials()`): the list files, the merged material
/// files and the names each list creates.
///
/// The client loads `materials/common.json` into one group and, by its graphics mode, either
/// `materials/fancy.json` (Fancy) or `materials/sad.json` (Simple, Vibrant Visuals and ray
/// tracing) into a second. Each list names `.material` files; every pack's copy of a named file
/// is merged as JSON, lowest pack first, and the materials of the merged file are created
/// parent first. A name is defined when the common group or either of the other two has it.
#[doc(alias = "materials")]
#[derive(Debug)]
#[non_exhaustive]
pub struct Materials {
    /// Every pack's list files, top pack first and within a pack by path: `common.json`,
    /// `fancy.json`, `gameface.json`, `sad.json`. `gameface.json` is read as a list and feeds no
    /// group: the client's material reload never asks for it.
    pub lists: Vec<Document<MaterialList>>,
    /// Every `.material` file of the stack by its path, each merged over the packs that have
    /// it; a file no list names is merged too and creates no material.
    pub files: IndexMap<PackPath, Merged<MaterialFile, RawLayer>>,
    /// The materials `materials/common.json` creates.
    pub common: MaterialGroup,
    /// The materials `materials/fancy.json` creates.
    pub fancy: MaterialGroup,
    /// The materials `materials/sad.json` creates.
    pub sad: MaterialGroup,
    /// The list files that did not load, top pack first and within a pack by path.
    pub(crate) rejected: Vec<Rejected>,
    /// What the load found outside the files' own findings.
    pub(crate) diagnostics: Diagnostics,
}

impl Materials {
    /// The group of `kind`.
    #[must_use]
    pub fn group(&self, kind: MaterialGroupKind) -> &MaterialGroup {
        match kind {
            MaterialGroupKind::Common => &self.common,
            MaterialGroupKind::Fancy => &self.fancy,
            MaterialGroupKind::Sad => &self.sad,
        }
    }

    /// The material `name`: the common group's, else the fancy group's, else the sad group's.
    /// A variant (`entity_static.skinning`) is no material of its own: ask
    /// [`contains`](Self::contains) or [`Material::has_variant`].
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Material> {
        self.common
            .get(name)
            .or_else(|| self.fancy.get(name))
            .or_else(|| self.sad.get(name))
    }

    /// Whether any group defines `id`, as a material or as a variant `<name>.<variant>`: what
    /// a reference from a client entity is checked against. The client checks its active
    /// groups only; a name one graphics mode lacks is still defined here.
    // A name counts when it is in the common group or in either graphics mode's group.
    #[must_use]
    pub fn contains(&self, id: &MaterialId) -> bool {
        let name = id.as_str();
        self.common.contains(name) || self.fancy.contains(name) || self.sad.contains(name)
    }

    /// The material `name` of the group `kind` with everything it inherits applied: the list
    /// entry's defines, then each ancestor's members from the root down, then its own. `None`
    /// when the group has no such material.
    ///
    /// The cost is the length of the parent chain, walked on every call: right for one name
    /// or a few. Resolving every material of a group this way is quadratic in a long chain;
    /// [`resolve_all`](Self::resolve_all) resolves them all at the cost of the group.
    #[must_use]
    pub fn resolve(&self, kind: MaterialGroupKind, name: &str) -> Option<ResolvedMaterial> {
        resolve::resolve(self, self.group(kind), name)
    }

    /// Every material of the group `kind` with everything it inherits applied, by name, in
    /// creation order: what [`resolve`](Self::resolve) gives for each.
    ///
    /// Each material is worked out from the state of its parent, which its siblings share, so
    /// the cost is the size of the group and of what is returned, whatever the depth of its
    /// parent chains. This is the call for many lookups: resolve the group once and look the
    /// names up in the map.
    ///
    /// What is returned can itself be large: each material holds the defines of its whole
    /// chain, so a long chain whose links each add a define returns a number of defines that
    /// is quadratic in the chain.
    #[must_use]
    pub fn resolve_all(&self, kind: MaterialGroupKind) -> IndexMap<String, ResolvedMaterial> {
        resolve::resolve_all(self, self.group(kind))
    }

    /// The list files that failed to load, top pack first, each with its findings. A list
    /// that is rejected feeds no group; the lists of the other packs are read as they are.
    #[must_use]
    pub fn rejected(&self) -> &[Rejected] {
        &self.rejected
    }

    /// Every finding of the load: the lists' and the files' own, then those of the list files
    /// that did not load (each also with its file in [`rejected`](Self::rejected)) and of
    /// creating the materials.
    pub fn diagnostics(&self) -> impl Iterator<Item = &Diagnostic> + '_ {
        let lists = self.lists.iter().flat_map(|list| list.diagnostics().iter());
        let files = self.files.values().flat_map(Merged::diagnostics);
        lists.chain(files).chain(self.diagnostics.iter())
    }
}
