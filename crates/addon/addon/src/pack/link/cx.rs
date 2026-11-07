//! The context of one link pass.

#![allow(
    dead_code,
    reason = "the frame of the rule packages: each uses the part its rows need"
)]

use std::cell::RefCell;
use std::collections::HashSet;

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, JsonPath, Location, Phase, Severity,
};
use bedrock_addon_core::load::{BuiltIns, StackPack};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bytes::Bytes;

use super::lookup::Caches;
use super::{Findings, LinkOptions};
use crate::pack::{Behavior, PackStack, Resource, Side, StackSet};
use crate::storage::StorageError;

/// One link pass over a world's two stacks: what every rule reads. The lookups are in
/// [`lookup`](super::lookup); each builds what it compares against on its first call and
/// keeps it for the pass, so a pass pays nothing for a family no rule asks about.
pub(crate) struct LinkCx<'a> {
    set: &'a StackSet,
    options: LinkOptions,
    /// `LoadOptions::built_ins` of the stacks.
    built_ins: Option<&'a dyn BuiltIns>,
    pub(super) caches: Caches<'a>,
    /// The files a rule asked the bytes of and that are not resident: each is reported once.
    not_loaded: RefCell<HashSet<(PackSide, usize, PackPath)>>,
}

impl std::fmt::Debug for LinkCx<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LinkCx")
            .field("options", &self.options)
            .field("built_ins", &self.built_ins.map(|_| ".."))
            .field("caches", &self.caches)
            .field("not_loaded", &self.not_loaded.borrow().len())
            .finish_non_exhaustive()
    }
}

impl<'a> LinkCx<'a> {
    pub(super) fn new(set: &'a StackSet, options: LinkOptions) -> Self {
        let built_ins =
            |options: &'a bedrock_addon_core::load::LoadOptions| options.built_ins.as_deref();
        Self {
            set,
            options,
            built_ins: built_ins(set.behavior.options()).or(built_ins(set.resource.options())),
            caches: Caches::default(),
            not_loaded: RefCell::default(),
        }
    }

    /// Both stacks.
    pub(crate) fn set(&self) -> &'a StackSet {
        self.set
    }

    /// The behaviour-pack stack.
    pub(crate) fn behavior(&self) -> &'a PackStack<Behavior> {
        &self.set.behavior
    }

    /// The resource-pack stack.
    pub(crate) fn resource(&self) -> &'a PackStack<Resource> {
        &self.set.resource
    }

    /// What the pass was asked to do.
    pub(crate) fn options(&self) -> LinkOptions {
        self.options
    }

    /// Whether the lints run. A rule that tells only informations may ask before it does any
    /// work; one that tells warnings too runs, and the sink drops its informations.
    pub(crate) fn lints(&self) -> bool {
        self.options.lints
    }

    /// Whether the stacks are linked as the game's client loads them, not as a dedicated
    /// server does. A rule of what only the client loads asks before it tells.
    pub(crate) fn client(&self) -> bool {
        self.options.client
    }

    /// The host's answer for the game's own content, when the stacks were built with one.
    pub(crate) fn host_built_ins(&self) -> Option<&'a dyn BuiltIns> {
        self.built_ins
    }

    /// The entry of `pack` on the stack of `side`: its load context and the files it lists by
    /// path, without their bytes (`files()`, `files_below(folder)`): how a rule knows which
    /// images, sounds or other binaries a pack has. `None` when `pack` is not the pack at
    /// its index of that stack.
    pub(crate) fn stack_pack(&self, side: PackSide, pack: &PackRef) -> Option<&'a StackPack> {
        let index = pack.index()?;
        let entry = match side {
            PackSide::Behavior => self.set.behavior.stack_packs().get(index),
            PackSide::Resource => self.set.resource.stack_packs().get(index),
        }?;
        (entry.pack == *pack).then_some(entry)
    }

    /// The bytes of the file `path` of `pack` on the stack of `side`, when the pack holds
    /// them: the pass issues no request for bytes. `None` for a file that is not resident, that
    /// the pack does not have or that cannot be read; a rule skips that file. A file that is
    /// indexed and not resident is told to `out` once per pass, as an information
    /// (`FileNotLoaded`), however many rules ask for it.
    // Only resident bytes are given: the pass never requests a file's bytes, so a rule skips a
    // file that is indexed and not loaded.
    pub(crate) fn bytes(
        &self,
        side: PackSide,
        pack: &PackRef,
        path: &PackPath,
        out: &mut Findings,
    ) -> Option<Bytes> {
        let index = pack.index()?;
        let held = match side {
            PackSide::Behavior => holder(&self.set.behavior, index, pack),
            PackSide::Resource => holder(&self.set.resource, index, pack),
        }?;
        match held.resident(path) {
            Ok(bytes) => Some(bytes),
            Err(StorageError::NotResident) => {
                let first = self
                    .not_loaded
                    .borrow_mut()
                    .insert((side, index, path.clone()));
                if first {
                    out.push(Diagnostic::new(
                        DiagnosticCode::from(&core_codes::FILE_NOT_LOADED),
                        Severity::Info,
                        Phase::Link,
                        Location::new(
                            Some(pack.clone()),
                            Some(path.clone()),
                            JsonPath::new(),
                            None,
                        ),
                    ));
                }
                None
            }
            Err(_) => None,
        }
    }
}

/// The pack `stack` has at `index`, when it is `pack`.
fn holder<'a, S: Side>(
    stack: &'a PackStack<S>,
    index: usize,
    pack: &PackRef,
) -> Option<&'a crate::pack::Pack> {
    let entry = stack.stack_packs().get(index)?;
    if entry.pack != *pack {
        return None;
    }
    stack.instances().get(index).map(|instance| &instance.pack)
}
