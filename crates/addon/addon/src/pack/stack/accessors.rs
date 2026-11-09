//! The stack accessors. The family list (`families/list.rs`) names every accessor once, in its
//! family's group; `family_accessors!` hands its lines, each with its family's feature, to
//! `accessors!`, which expands them to the cache fields (behind each feature), the accessor
//! methods on `PackStack`, the `load::Collection` enum and the table the `StackLoader` drives.
//!
//! An accessor line, as `accessors!` takes it, reads
//! `blocks: Registry<Block> = block::LOADER, collection = Blocks, feature = "block", side = Behavior;`:
//! the method name and the collection it returns, the module's loader (a
//! `const LOADER: load::CollectionLoader<C, P>`, the per-collection loader), the
//! line's `Collection` variant, the content feature that gates it and the stack
//! side it exists on. The method drives the loader on first call (walk, parse per file,
//! combine with the stack's `StackCx`) and caches the collection; later calls return the same
//! reference. A `StackLoader` fills the same cache through the line's entry in the table.
//!
//! Three more forms, each also written with any of the two sides:
//!
//! - **Both sides**: `side = Both` gives one generic accessor in `impl<S: Side> PackStack<S>`
//!   (`animations`, `languages`), backed by one cache per side. Its loader is the same on both
//!   sides; each walk runs over the packs of its own stack, and a file carries the side it was
//!   read on (`WalkFile::side`). The line has one `Collection` variant, shared by the two
//!   entries of the table (one per side): selecting it loads the collection on both stacks.
//! - **Keyed**: `translations[language: Language]: Merged<Translations> = lang::LOADER,
//!   keys = listed_languages, key_of = lang::language_of, collection = Translations,
//!   side = Both;` gives `fn translations(&self, language: &Language) ->
//!   Option<&Merged<Translations>>`. `keys` (`fn(&PackStack<S>) -> impl IntoIterator<Item = K>`,
//!   for `translations` the codes `languages()` lists) is called once, on the first call; a key
//!   it does not give is `None`. `key_of` (`fn(&PackPath) -> Option<K>`) names the key a walked
//!   file belongs to. Each listed key loads on its own first call (walk, the key's files only,
//!   parse, combine) into its own `OnceLock`. A full load (`StackLoader`, `load_all`) parses
//!   every walked file with a key, listed or not, and combines each key's files; the accessor
//!   then answers a listed key from it, and still `None` for the others.
//! - **Slice**: `scripts: [ScriptBundle] = script::LOADER, …;` caches the loader's
//!   `Vec<ScriptBundle>` and returns `&[ScriptBundle]`.
//!
//! The list is expanded one line at a time (a recursive macro), and each line costs two levels
//! of the crate's recursion limit (`@munch` takes the line to `@line`, which hands the rest back
//! to `@munch`). The 58 lines of the family list take some 120 levels; `lib.rs` sets
//! `recursion_limit = "512"`, room for about 250.

use std::fmt;
use std::hash::Hash;
use std::sync::OnceLock;

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics};
use bedrock_addon_core::load::{CollectionLoader, LoadContext, Reported, StackCx, Walk, WalkFile};
use bedrock_addon_core::pack_identity::{PackPath, PackSide};
use indexmap::IndexMap;

use super::{PackStack, Side, read_item, walk_items};
use crate::families::with_families;
use crate::pack::stack::{Erased, Loaded};
// What the accessor lines name: the document collections and the family modules, which
// are whichever families the features compile in.
#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "the lines name every family compiled in, and none of a family compiled out"
)]
use crate::{
    document::{Files, Merged, RawLayer, Registry},
    families::*,
};

/// The cache of a line on the side it is read from: a one-side cache is itself, a both-sides
/// cache ([`BothSides`]) holds one per side.
#[allow(
    dead_code,
    reason = "the accessor lines use it; the list fills as the families land"
)]
pub(crate) trait Sided {
    /// The cache of one side.
    type Cache;
    /// The cache of `side`.
    fn of(&self, side: PackSide) -> &Self::Cache;
}

impl<C> Sided for OnceLock<Loaded<C>> {
    type Cache = Self;
    fn of(&self, _: PackSide) -> &Self {
        self
    }
}

impl<K, C> Sided for Keyed<K, C> {
    type Cache = Self;
    fn of(&self, _: PackSide) -> &Self {
        self
    }
}

/// The two caches of a `side = Both` line.
#[derive(Default)]
#[allow(
    dead_code,
    reason = "the accessor lines use it; the list fills as the families land"
)]
pub(crate) struct BothSides<T> {
    behavior: T,
    resource: T,
}

impl<T> Sided for BothSides<T> {
    type Cache = T;
    fn of(&self, side: PackSide) -> &T {
        match side {
            PackSide::Behavior => &self.behavior,
            PackSide::Resource => &self.resource,
        }
    }
}

/// The collection in `cache`, loaded through `loader` on first call.
#[allow(
    dead_code,
    reason = "the accessor lines use it; the list fills as the families land"
)]
pub(crate) fn load_once<'a, S: Side, C, P>(
    cache: &'a OnceLock<Loaded<C>>,
    loader: CollectionLoader<C, P>,
    stack: &PackStack<S>,
) -> &'a C {
    &cache
        .get_or_init(|| {
            let (value, findings) = stack.drive(loader);
            Loaded { value, findings }
        })
        .value
}

/// The cache of a keyed line: what a full load combined (every key with files) and one lazily
/// loaded value per listed key.
#[allow(
    dead_code,
    reason = "the accessor lines use it; the list fills as the families land"
)]
pub(crate) struct Keyed<K, C> {
    /// Every key the walk reached a file of, in walk order, from a `StackLoader` or `load_all`.
    full: OnceLock<Loaded<IndexMap<K, C>>>,
    /// The listed keys, in `keys` order, each loaded on its first call.
    listed: OnceLock<IndexMap<K, OnceLock<Loaded<C>>>>,
}

impl<K, C> Default for Keyed<K, C> {
    fn default() -> Self {
        Self {
            full: OnceLock::new(),
            listed: OnceLock::new(),
        }
    }
}

#[allow(
    dead_code,
    reason = "the accessor lines use it; the list fills as the families land"
)]
impl<K: Eq + Hash, C> Keyed<K, C> {
    /// The collection of `key`: `None` when `keys` does not give it; else the value loaded for
    /// it already (by an earlier call, else by a full load), else loaded now from its files.
    pub(crate) fn get<'a, S: Side, P, I: IntoIterator<Item = K>>(
        &'a self,
        key: &K,
        keys: impl FnOnce() -> I,
        key_of: fn(&PackPath) -> Option<K>,
        loader: CollectionLoader<C, P>,
        stack: &PackStack<S>,
    ) -> Option<&'a C> {
        let listed = self.listed.get_or_init(|| {
            keys()
                .into_iter()
                .map(|key| (key, OnceLock::new()))
                .collect()
        });
        let slot = listed.get(key)?;
        if let Some(loaded) = slot.get() {
            return Some(&loaded.value);
        }
        if let Some(value) = self.full.get().and_then(|full| full.value.get(key)) {
            return Some(value);
        }
        let own = slot.get_or_init(|| {
            let mut parsed = Vec::new();
            let findings = visit(
                &loader.walk,
                stack,
                |path| key_of(path).as_ref() == Some(key),
                |file, cx| parsed.push((loader.parse)(file, cx)),
            );
            let value =
                (loader.combine)(parsed, &StackCx::new(stack.options(), stack.stack_packs()));
            Loaded { value, findings }
        });
        Some(&own.value)
    }

    /// Loads every key the walk reaches a file of, unless loaded already.
    pub(crate) fn load_all<S: Side, P>(
        &self,
        key_of: fn(&PackPath) -> Option<K>,
        loader: CollectionLoader<C, P>,
        stack: &PackStack<S>,
    ) {
        self.full.get_or_init(|| {
            let mut parsed = Vec::new();
            let findings = visit(
                &loader.walk,
                stack,
                |path| key_of(path).is_some(),
                |file, cx| parsed.push(keyed_parse(loader, key_of, file, cx)),
            );
            let cx = StackCx::new(stack.options(), stack.stack_packs());
            Loaded {
                value: keyed_combine(loader, parsed.into_iter().flatten(), &cx),
                findings,
            }
        });
    }

    /// Fills the full load with a `StackLoader`'s combined value, unless loaded already.
    pub(crate) fn install(&self, value: Erased, findings: Diagnostics)
    where
        K: 'static,
        C: 'static,
    {
        if let Ok(value) = value.downcast::<IndexMap<K, C>>() {
            let _ = self.full.set(Loaded {
                value: *value,
                findings,
            });
        }
    }

    /// A full load happened.
    pub(crate) fn is_loaded(&self) -> bool {
        self.full.get().is_some()
    }

    /// What is loaded, key by key: the full load's keys in walk order, then the listed keys
    /// loaded on their own, each with the loader findings of its own load.
    fn loaded(&self) -> impl Iterator<Item = (&K, &C, Option<&Diagnostics>)> + '_ {
        let full = self.full.get();
        let full_keys = full
            .into_iter()
            .flat_map(|full| full.value.iter().map(|(key, value)| (key, value, None)));
        let own = self.listed.get().into_iter().flat_map(move |listed| {
            listed.iter().filter_map(move |(key, slot)| {
                let loaded = slot.get()?;
                let in_full = full.is_some_and(|full| full.value.contains_key(key));
                (!in_full).then_some((key, &loaded.value, Some(&loaded.findings)))
            })
        });
        full_keys.chain(own)
    }

    /// The full load's findings, then per loaded key its own findings and its diagnostics.
    pub(crate) fn report<'a>(&'a self, out: &mut Vec<&'a Diagnostic>)
    where
        C: Reported,
    {
        if let Some(full) = self.full.get() {
            out.extend(full.findings.iter());
        }
        for (_, value, findings) in self.loaded() {
            out.extend(findings.into_iter().flat_map(Diagnostics::iter));
            out.extend(value.reported());
        }
    }

    /// The loaded keys and their collections, `None` when nothing is loaded.
    pub(crate) fn debug(&self) -> Option<String>
    where
        K: fmt::Debug,
        C: fmt::Debug,
    {
        let loaded: Vec<(&K, &C)> = self.loaded().map(|(key, value, _)| (key, value)).collect();
        (!loaded.is_empty() || self.is_loaded()).then(|| format!("{loaded:?}"))
    }
}

/// A keyed line's `Parse` job: the file's key and its parsed value; nothing for a file without
/// a key.
#[allow(
    dead_code,
    reason = "the accessor lines use it; the list fills as the families land"
)]
fn keyed_parse<K, C, P>(
    loader: CollectionLoader<C, P>,
    key_of: fn(&PackPath) -> Option<K>,
    file: &WalkFile,
    cx: &LoadContext,
) -> Option<(K, P)> {
    key_of(&file.key).map(|key| (key, (loader.parse)(file, cx)))
}

/// A keyed line's `Combine` job: each key's files, in walk order, combined.
#[allow(
    dead_code,
    reason = "the accessor lines use it; the list fills as the families land"
)]
fn keyed_combine<K: Eq + Hash, C, P>(
    loader: CollectionLoader<C, P>,
    parsed: impl IntoIterator<Item = (K, P)>,
    cx: &StackCx<'_>,
) -> IndexMap<K, C> {
    let mut groups: IndexMap<K, Vec<P>> = IndexMap::new();
    for (key, value) in parsed {
        groups.entry(key).or_default().push(value);
    }
    groups
        .into_iter()
        .map(|(key, files)| (key, (loader.combine)(files, cx)))
        .collect()
}

/// [`keyed_parse`], type-erased for the `StackLoader`.
#[allow(
    dead_code,
    reason = "the accessor lines use it; the list fills as the families land"
)]
pub(crate) fn erased_keyed_parse<K: Send + Sync + 'static, C, P: Send + Sync + 'static>(
    loader: CollectionLoader<C, P>,
    key_of: fn(&PackPath) -> Option<K>,
    file: &WalkFile,
    cx: &LoadContext,
) -> Erased {
    Box::new(keyed_parse(loader, key_of, file, cx))
}

/// [`keyed_combine`], type-erased for the `StackLoader`; `key_of` only names `K`.
#[allow(
    dead_code,
    reason = "the accessor lines use it; the list fills as the families land"
)]
pub(crate) fn erased_keyed_combine<
    K: Eq + Hash + Send + Sync + 'static,
    C: Send + Sync + 'static,
    P: 'static,
>(
    loader: CollectionLoader<C, P>,
    _key_of: fn(&PackPath) -> Option<K>,
    parsed: Vec<Erased>,
    cx: &StackCx<'_>,
) -> Erased {
    let parsed = parsed.into_iter().filter_map(|value| {
        value
            .downcast::<Option<(K, P)>>()
            .ok()
            .and_then(|value| *value)
    });
    Box::new(keyed_combine(loader, parsed, cx))
}

/// Walks `stack` as `PackStack::drive` does, keeping the files whose key `keep` accepts, and
/// hands each to `each` with its pack's context; returns the read findings.
// The lazy path of `PackStack::drive` with a key filter, so a keyed line reads only one key's
// files.
#[allow(
    dead_code,
    reason = "the accessor lines use it; the list fills as the families land"
)]
fn visit<S: Side>(
    walk: &Walk,
    stack: &PackStack<S>,
    keep: impl Fn(&PackPath) -> bool,
    mut each: impl FnMut(&WalkFile, &LoadContext),
) -> Diagnostics {
    let entries = stack.listed_files();
    let app_data = stack.options().app_data.as_ref();
    let app_data_context = stack.app_data_context();
    let mut findings = Diagnostics::default();
    for item in walk_items(walk, &entries, stack.stack_packs(), app_data) {
        if !keep(&item.key) {
            continue;
        }
        match read_item(stack, item, app_data, &app_data_context) {
            Ok(Some((file, context))) => each(&file, &context),
            Ok(None) => {}
            Err(finding) => findings.push(*finding),
        }
    }
    findings
}

macro_rules! accessors {
    // ---- entry points ----
    // The accessor list itself, over `PackStack`. A line starts with its doc comment or its
    // name; anything else (a mistyped `@` form) matches no rule.
    () => {
        accessors! { @munch [list Caches PackStack real] [] }
    };
    (# $($lines:tt)*) => {
        accessors! { @munch [list Caches PackStack real] [] # $($lines)* }
    };
    ($name:ident $($lines:tt)*) => {
        accessors! { @munch [list Caches PackStack real] [] $name $($lines)* }
    };
    // The caches and the methods over a host (the macro test's toy host).
    (@host $caches:ident for $host:ident; $($lines:tt)*) => {
        accessors! { @munch [host $caches $host] [] $($lines)* }
    };
    // Everything over a host: also `Collection`, the entries and `ENTRIES` (a test list: its
    // entries' `collection()` is `None`, `ENTRY_COLLECTIONS` names their variants).
    (@list $caches:ident for $host:ident; $($lines:tt)*) => {
        accessors! { @munch [list $caches $host test] [] $($lines)* }
    };

    // ---- one line at a time into one shape ----
    // `{ name, [docs], variant, (feature?), side, [entries], loader, (cache type), (form …) }`;
    // an entry is `(variant, side, (feature?))`, two for `side = Both`.
    (@munch [host $caches:ident $host:ident] [$($done:tt)*]) => {
        accessors! { @emit_host $caches $host [$($done)*] }
    };
    (@munch [list $caches:ident $host:ident $kind:ident] [$($done:tt)*]) => {
        accessors! { @emit_host $caches $host [$($done)*] }
        accessors! { @emit_list $kind $host [$($done)*] }
    };
    // Keyed.
    (@munch $mode:tt [$($done:tt)*]
        $(#[doc = $doc:literal])*
        $name:ident [$param:ident : $key:ty] : $collection:ty = $loader:path,
            keys = $keys:path, key_of = $key_of:path, collection = $variant:ident
            $(, feature = $feature:literal)?, side = $side:ident;
        $($rest:tt)*
    ) => {
        accessors! { @line $mode [$($done)*] $side {
            $name, [$(#[doc = $doc])*], $variant, ($($feature)?), $loader,
            ($crate::pack::stack::accessors::Keyed<$key, $collection>),
            (keyed $param, $key, $keys, $key_of, $collection)
        } $($rest)* }
    };
    // Slice.
    (@munch $mode:tt [$($done:tt)*]
        $(#[doc = $doc:literal])*
        $name:ident : [$element:ty] = $loader:path, collection = $variant:ident
            $(, feature = $feature:literal)?, side = $side:ident;
        $($rest:tt)*
    ) => {
        accessors! { @line $mode [$($done)*] $side {
            $name, [$(#[doc = $doc])*], $variant, ($($feature)?), $loader,
            (::std::sync::OnceLock<$crate::pack::stack::Loaded<::std::vec::Vec<$element>>>),
            (slice $element)
        } $($rest)* }
    };
    // One collection.
    (@munch $mode:tt [$($done:tt)*]
        $(#[doc = $doc:literal])*
        $name:ident : $collection:ty = $loader:path, collection = $variant:ident
            $(, feature = $feature:literal)?, side = $side:ident;
        $($rest:tt)*
    ) => {
        accessors! { @line $mode [$($done)*] $side {
            $name, [$(#[doc = $doc])*], $variant, ($($feature)?), $loader,
            (::std::sync::OnceLock<$crate::pack::stack::Loaded<$collection>>),
            (plain $collection)
        } $($rest)* }
    };
    // A parsed line into the shape, with its entries: two for `side = Both`, else one.
    (@line $mode:tt [$($done:tt)*] Both {
        $name:ident, $docs:tt, $variant:ident, $feature:tt, $loader:path, $cache:tt, $form:tt
    } $($rest:tt)*) => {
        accessors! { @munch $mode [$($done)* {
            $name, $docs, $variant, $feature, Both,
            [($variant, Behavior, $feature) ($variant, Resource, $feature)],
            $loader, $cache, $form
        }] $($rest)* }
    };
    (@line $mode:tt [$($done:tt)*] $side:ident {
        $name:ident, $docs:tt, $variant:ident, $feature:tt, $loader:path, $cache:tt, $form:tt
    } $($rest:tt)*) => {
        accessors! { @munch $mode [$($done)* {
            $name, $docs, $variant, $feature, $side, [($variant, $side, $feature)],
            $loader, $cache, $form
        }] $($rest)* }
    };

    // ---- the caches and the methods ----
    (@emit_host $caches:ident $host:ident [$(
        {
            $name:ident, $docs:tt, $variant:ident, ($($feature:literal)?), $side:ident,
            $entries:tt, $loader:path, $cache:tt, $form:tt
        }
    )*]) => {
        /// The cached collections of a stack, one `OnceLock` per accessor and side (a keyed
        /// accessor: one per key).
        #[derive(Default)]
        pub(crate) struct $caches {
            $(
                $(#[cfg(feature = $feature)])?
                $name: accessors!(@sided $side $cache),
            )*
        }

        $(
            accessors! { @method $host, $side, $name, $docs, ($($feature)?), $loader, $form }
        )*
    };
    (@sided Both ($($cache:tt)*)) => {
        $crate::pack::stack::accessors::BothSides<$($cache)*>
    };
    (@sided $side:ident ($($cache:tt)*)) => {
        $($cache)*
    };
    (@method $host:ident, Both, $($line:tt)*) => {
        accessors! {
            @method_body [impl<S: $crate::pack::Side> $host<S>] (S) $($line)*
        }
    };
    (@method $host:ident, $side:ident, $($line:tt)*) => {
        accessors! {
            @method_body [impl $host<$crate::pack::$side>] ($crate::pack::$side) $($line)*
        }
    };
    (@method_body [$($impl:tt)*] $side:tt $name:ident, [$(#[doc = $doc:literal])*],
        ($($feature:literal)?), $loader:path, (keyed $param:ident, $key:ty, $keys:path,
        $key_of:path, $collection:ty)
    ) => {
        $(#[cfg(feature = $feature)])?
        $(#[cfg_attr(docsrs, doc(cfg(feature = $feature)))])?
        $($impl)* {
            $(#[doc = $doc])*
            pub fn $name(&self, $param: &$key) -> ::std::option::Option<&$collection> {
                let side = <accessors!(@type $side) as $crate::pack::Side>::SIDE;
                $crate::pack::stack::accessors::Sided::of(&self.caches.$name, side).get(
                    $param,
                    || $keys(self),
                    $key_of,
                    $loader,
                    self.as_stack(),
                )
            }
        }
    };
    (@method_body [$($impl:tt)*] $side:tt $name:ident, [$(#[doc = $doc:literal])*],
        ($($feature:literal)?), $loader:path, (slice $element:ty)
    ) => {
        $(#[cfg(feature = $feature)])?
        $(#[cfg_attr(docsrs, doc(cfg(feature = $feature)))])?
        $($impl)* {
            $(#[doc = $doc])*
            pub fn $name(&self) -> &[$element] {
                let side = <accessors!(@type $side) as $crate::pack::Side>::SIDE;
                $crate::pack::stack::accessors::load_once(
                    $crate::pack::stack::accessors::Sided::of(&self.caches.$name, side),
                    $loader,
                    self.as_stack(),
                )
                .as_slice()
            }
        }
    };
    (@method_body [$($impl:tt)*] $side:tt $name:ident, [$(#[doc = $doc:literal])*],
        ($($feature:literal)?), $loader:path, (plain $collection:ty)
    ) => {
        $(#[cfg(feature = $feature)])?
        $(#[cfg_attr(docsrs, doc(cfg(feature = $feature)))])?
        $($impl)* {
            $(#[doc = $doc])*
            pub fn $name(&self) -> &$collection {
                let side = <accessors!(@type $side) as $crate::pack::Side>::SIDE;
                $crate::pack::stack::accessors::load_once(
                    $crate::pack::stack::accessors::Sided::of(&self.caches.$name, side),
                    $loader,
                    self.as_stack(),
                )
            }
        }
    };
    // The side type of a line, as `@method` and `@entry` give it: `(S)` for both sides.
    (@type ($($side:tt)*)) => {
        $($side)*
    };

    // ---- `Collection`, the entries and `ENTRIES` ----
    (@emit_list $kind:ident $host:ident [$(
        {
            $name:ident, [$(#[doc = $doc:literal])*], $variant:ident, ($($feature:literal)?),
            $side:ident, [$(($entry:ident, $entry_side:ident, ($($entry_feature:literal)?)))*],
            $loader:path, $cache:tt, $form:tt
        }
    )*]) => {
        /// One variant per stack accessor; generated by the accessor list, so the list exists
        /// once. A both-sides accessor is one variant: selecting it loads both stacks'.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        #[non_exhaustive]
        pub enum Collection {
            $(
                $(#[cfg(feature = $feature)])?
                $(#[cfg_attr(docsrs, doc(cfg(feature = $feature)))])?
                $(#[doc = $doc])*
                $variant,
            )*
        }

        impl Collection {
            /// Every collection compiled in, in list order.
            pub(crate) const ALL: &'static [Collection] = &[
                $( $(#[cfg(feature = $feature)])? Collection::$variant, )*
            ];
        }

        /// The `StackLoader`'s view of each line: one type per line, used once per side.
        mod entry {
            #[allow(unused_imports, reason = "the accessor lines use them")]
            use super::*;
            $(
                accessors! {
                    @entry $kind $host, $side, $name, $variant, ($($feature)?), $loader, $form
                }
            )*
        }

        /// One entry per accessor and side compiled in, in list order (behaviour side first):
        /// what a `StackLoader` drives.
        pub(crate) static ENTRIES: &[&dyn $crate::pack::stack::ErasedCollection] = &[
            $($(
                $(#[cfg(feature = $entry_feature)])?
                &entry::$entry::<$crate::pack::$entry_side>(::std::marker::PhantomData),
            )*)*
        ];

        accessors! {
            @entry_collections $kind [$($( ($entry, ($($entry_feature)?)) )*)*]
        }
    };
    (@entry_collections real $($entries:tt)*) => {};
    (@entry_collections test [$(($entry:ident, ($($feature:literal)?)))*]) => {
        /// The `Collection` of each entry of a test list, aligned with `ENTRIES`.
        pub(crate) static ENTRY_COLLECTIONS: &[Collection] = &[
            $( $(#[cfg(feature = $feature)])? Collection::$entry, )*
        ];
    };
    // An entry's `collection()`: `None` for a test list, whose `Collection` is its own.
    (@collection real $variant:ident) => {
        Some(Collection::$variant)
    };
    (@collection test $variant:ident) => {
        None
    };
    (@entry $kind:ident $host:ident, Both, $name:ident, $variant:ident, $($line:tt)*) => {
        accessors! {
            @entry_body [impl<S: $crate::pack::Side> $crate::pack::stack::ErasedCollection for $variant<S>]
            (S) $kind $host, $name, $variant, $($line)*
        }
    };
    (@entry $kind:ident $host:ident, $side:ident, $name:ident, $variant:ident, $($line:tt)*) => {
        accessors! {
            @entry_body
            [impl $crate::pack::stack::ErasedCollection for $variant<$crate::pack::$side>]
            ($crate::pack::$side) $kind $host, $name, $variant, $($line)*
        }
    };
    (@entry_body [$($impl:tt)*] $side:tt $kind:ident $host:ident, $name:ident, $variant:ident,
        ($($feature:literal)?), $loader:path, (keyed $param:ident, $key:ty, $keys:path,
        $key_of:path, $collection:ty)
    ) => {
        $(#[cfg(feature = $feature)])?
        pub(crate) struct $variant<S>(pub(crate) ::std::marker::PhantomData<fn() -> S>);

        $(#[cfg(feature = $feature)])?
        $($impl)* {
            fn collection(&self) -> ::std::option::Option<$crate::pack::stack::Collection> {
                accessors!(@collection $kind $variant)
            }
            fn side(&self) -> $crate::pack::PackSide {
                <accessors!(@type $side) as $crate::pack::Side>::SIDE
            }
            fn walk(&self) -> ::bedrock_addon_core::load::Walk {
                $loader.walk
            }
            fn needs(&self) -> &'static [&'static str] {
                $loader.needs
            }
            fn parse(
                &self,
                file: &::bedrock_addon_core::load::WalkFile,
                cx: &::bedrock_addon_core::load::LoadContext,
            ) -> $crate::pack::stack::Erased {
                $crate::pack::stack::accessors::erased_keyed_parse(
                    $loader,
                    $key_of,
                    file,
                    cx,
                )
            }
            fn combine(
                &self,
                parsed: Vec<$crate::pack::stack::Erased>,
                cx: &::bedrock_addon_core::load::StackCx<'_>,
            ) -> $crate::pack::stack::Erased {
                $crate::pack::stack::accessors::erased_keyed_combine(
                    $loader,
                    $key_of,
                    parsed,
                    cx,
                )
            }
            fn install(
                &self,
                stack: &dyn ::std::any::Any,
                value: $crate::pack::stack::Erased,
                findings: ::bedrock_addon_core::diagnostic::Diagnostics,
            ) {
                if let Some(stack) = stack.downcast_ref::<$host<accessors!(@type $side)>>() {
                    $crate::pack::stack::accessors::Sided::of(&stack.caches.$name, self.side())
                        .install(value, findings);
                }
            }
            fn is_loaded(&self, stack: &dyn ::std::any::Any) -> bool {
                stack.downcast_ref::<$host<accessors!(@type $side)>>().is_some_and(|stack| {
                    $crate::pack::stack::accessors::Sided::of(&stack.caches.$name, self.side())
                        .is_loaded()
                })
            }
            fn load(&self, stack: &dyn ::std::any::Any) {
                if let Some(stack) = stack.downcast_ref::<$host<accessors!(@type $side)>>() {
                    $crate::pack::stack::accessors::Sided::of(&stack.caches.$name, self.side())
                        .load_all($key_of, $loader, stack.as_stack());
                }
            }
            fn report<'a>(
                &self,
                stack: &'a dyn ::std::any::Any,
                out: &mut Vec<&'a ::bedrock_addon_core::diagnostic::Diagnostic>,
            ) {
                if let Some(stack) = stack.downcast_ref::<$host<accessors!(@type $side)>>() {
                    $crate::pack::stack::accessors::Sided::of(&stack.caches.$name, self.side())
                        .report(out);
                }
            }
            fn debug(&self, stack: &dyn ::std::any::Any) -> Option<String> {
                let stack = stack.downcast_ref::<$host<accessors!(@type $side)>>()?;
                $crate::pack::stack::accessors::Sided::of(&stack.caches.$name, self.side())
                    .debug()
            }
        }
    };
    (@entry_body [$($impl:tt)*] $side:tt $kind:ident $host:ident, $name:ident, $variant:ident,
        ($($feature:literal)?), $loader:path, $form:tt
    ) => {
        $(#[cfg(feature = $feature)])?
        pub(crate) struct $variant<S>(pub(crate) ::std::marker::PhantomData<fn() -> S>);

        $(#[cfg(feature = $feature)])?
        $($impl)* {
            fn collection(&self) -> ::std::option::Option<$crate::pack::stack::Collection> {
                accessors!(@collection $kind $variant)
            }
            fn side(&self) -> $crate::pack::PackSide {
                <accessors!(@type $side) as $crate::pack::Side>::SIDE
            }
            fn walk(&self) -> ::bedrock_addon_core::load::Walk {
                $loader.walk
            }
            fn needs(&self) -> &'static [&'static str] {
                $loader.needs
            }
            fn registered(
                &self,
                stack: &dyn ::std::any::Any,
            ) -> ::std::option::Option<::bedrock_addon_core::load::Registered> {
                let loaded = $crate::pack::stack::accessors::Sided::of(
                    &stack.downcast_ref::<$host<accessors!(@type $side)>>()?.caches.$name,
                    self.side(),
                )
                .get()?;
                ::bedrock_addon_core::load::Reported::registered(&loaded.value)
            }
            fn has_then(&self) -> bool {
                $loader.then.is_some()
            }
            fn then(
                &self,
                parsed: &[$crate::pack::stack::Erased],
            ) -> Vec<::bedrock_addon_core::pack_identity::PackPath> {
                $crate::pack::stack::erased_then($loader, parsed)
            }
            fn parse(
                &self,
                file: &::bedrock_addon_core::load::WalkFile,
                cx: &::bedrock_addon_core::load::LoadContext,
            ) -> $crate::pack::stack::Erased {
                $crate::pack::stack::erased_parse($loader, file, cx)
            }
            fn combine(
                &self,
                parsed: Vec<$crate::pack::stack::Erased>,
                cx: &::bedrock_addon_core::load::StackCx<'_>,
            ) -> $crate::pack::stack::Erased {
                $crate::pack::stack::erased_combine($loader, parsed, cx)
            }
            fn install(
                &self,
                stack: &dyn ::std::any::Any,
                value: $crate::pack::stack::Erased,
                findings: ::bedrock_addon_core::diagnostic::Diagnostics,
            ) {
                if let Some(stack) = stack.downcast_ref::<$host<accessors!(@type $side)>>() {
                    $crate::pack::stack::install(
                        $crate::pack::stack::accessors::Sided::of(
                            &stack.caches.$name,
                            self.side(),
                        ),
                        value,
                        findings,
                    );
                }
            }
            fn is_loaded(&self, stack: &dyn ::std::any::Any) -> bool {
                stack.downcast_ref::<$host<accessors!(@type $side)>>().is_some_and(|stack| {
                    $crate::pack::stack::accessors::Sided::of(&stack.caches.$name, self.side())
                        .get()
                        .is_some()
                })
            }
            fn load(&self, stack: &dyn ::std::any::Any) {
                if let Some(stack) = stack.downcast_ref::<$host<accessors!(@type $side)>>() {
                    let _ = stack.$name();
                }
            }
            fn report<'a>(
                &self,
                stack: &'a dyn ::std::any::Any,
                out: &mut Vec<&'a ::bedrock_addon_core::diagnostic::Diagnostic>,
            ) {
                let loaded = stack.downcast_ref::<$host<accessors!(@type $side)>>().and_then(|stack| {
                    $crate::pack::stack::accessors::Sided::of(&stack.caches.$name, self.side())
                        .get()
                });
                if let Some(loaded) = loaded {
                    out.extend(loaded.findings.iter());
                    out.extend(::bedrock_addon_core::load::Reported::reported(&loaded.value));
                }
            }
            fn debug(&self, stack: &dyn ::std::any::Any) -> Option<String> {
                let loaded = $crate::pack::stack::accessors::Sided::of(
                    &stack.downcast_ref::<$host<accessors!(@type $side)>>()?.caches.$name,
                    self.side(),
                )
                .get()?;
                Some(format!("{:?}", loaded.value))
            }
        }
    };
}

/// The accessor lines of the family list ([`with_families!`]), each with its family's feature,
/// handed to [`accessors!`] in list order.
macro_rules! family_accessors {
    ($($family:literal $module:ident = $krate:ident $(:: $path:ident)* {
        $(
            $(#[doc = $doc:literal])*
            $name:ident $([$param:ident : $key:ty])?
                : $([$element:ty])? $($($segment:ident)::+ $(<$($argument:ty),+>)?)?
                = $loader:path, $(keys = $keys:path, key_of = $key_of:path,)?
                collection = $variant:ident, side = $side:ident;
        )*
    } $(lookups { $($lookups:tt)* })?)*) => {
        accessors! {
            $($(
                $(#[doc = $doc])*
                $name $([$param : $key])?
                    : $([$element])? $($($segment)::+ $(<$($argument),+>)?)?
                    = $loader, $(keys = $keys, key_of = $key_of,)?
                    collection = $variant, feature = $family, side = $side;
            )*)*
        }
    };
}

// The accessor list: the lines of the family list. A new family adds its lines to its group in
// `families/list.rs`.
with_families!(family_accessors);

/// The collection that registers each identifier kind another collection's `combine` may ask
/// the stack about (`StackCx::registered`): a `CollectionLoader::needs` entry names a kind, and
/// this list says which accessor answers for it. A kind whose family is not compiled in has no
/// line, and is then out of view (`Registration::NotInView`).
///
/// No cycles: a collection listed here needs nothing itself, which the assertion each line
/// generates holds at compile time, so the collection a `combine` waits for never waits for
/// another. The kind of a line is its loader's (`Registry<T>`'s `T::Kind`), so a line cannot
/// name a kind its collection does not register; that its variant is its loader's accessor is
/// `providers_name_their_loader_s_accessor` (tests).
macro_rules! providers {
    ($($feature:literal: $variant:ident = $loader:path;)*) => {
        pub(crate) const PROVIDERS: &[(&str, Collection)] = &[
            $(
                #[cfg(feature = $feature)]
                (registered_kind($loader), Collection::$variant),
            )*
        ];
        /// Each line's loader, by its variant, for the test that ties the two.
        #[cfg(test)]
        pub(crate) fn provider_walks() -> Vec<(Collection, Walk)> {
            vec![$(
                #[cfg(feature = $feature)]
                (Collection::$variant, $loader.walk),
            )*]
        }
        $(
            #[cfg(feature = $feature)]
            const _: () = assert!(
                $loader.needs.is_empty(),
                "a collection that registers a kind for others needs none itself"
            );
        )*
    };
}

/// The identifier kind a registry loader registers.
#[cfg(any(feature = "block", feature = "item", feature = "function"))]
const fn registered_kind<T: crate::document::Definition, P>(
    _: CollectionLoader<crate::document::Registry<T>, P>,
) -> &'static str {
    <T::Kind as crate::id::Kind>::NAME
}

providers! {
    "block": Blocks = block::LOADER;
    "item": Items = item::LOADER;
    "function": Functions = function::LOADER;
}

/// The line of `entries` on `side` that registers the identifiers of `kind`, by [`PROVIDERS`].
pub(crate) fn provider(
    kind: &str,
    side: PackSide,
    entries: &[&'static dyn crate::pack::stack::ErasedCollection],
) -> Option<&'static dyn crate::pack::stack::ErasedCollection> {
    let (_, collection) = PROVIDERS.iter().find(|(name, _)| *name == kind)?;
    entries
        .iter()
        .copied()
        .find(|entry| entry.collection() == Some(*collection) && entry.side() == side)
}

/// The keys of `translations`: the codes `languages()` lists, in order. Here, not in `lang`,
/// because a family never names another. A stack without the `languages` family lists every
/// code that has a `texts/<code>.lang` file in one of its packs.
#[cfg(feature = "lang")]
fn listed_languages<S: Side>(stack: &PackStack<S>) -> Vec<lang::Language> {
    #[cfg(feature = "languages")]
    {
        stack.languages().value().codes.clone()
    }
    #[cfg(not(feature = "languages"))]
    {
        let mut codes: Vec<lang::Language> = Vec::new();
        for pack in stack.stack_packs() {
            for file in pack.files_below("texts") {
                if let Some(code) = lang::language_of(&file.key)
                    && !codes.contains(&code)
                {
                    codes.push(code);
                }
            }
        }
        codes
    }
}

#[cfg(test)]
mod tests;
