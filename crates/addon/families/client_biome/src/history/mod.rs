//! The client-biome document as written in older format versions, and the steps that upgrade
//! it to [`super::ClientBiome`].
//!
//! The chain is `v1_21_40 → v1_21_70 → v1_21_90 → v1_21_100 → v1_21_110 → v1_21_120 →
//! v1_21_130`; each `vX_Y_Z::ClientBiome` parses `[X.Y.Z, next)` and its `Upgrade` impl is that
//! version's step. A document from 1.21.130 up to the engine version parses as
//! [`super::ClientBiome`]; `"format_version": "beta"` parses as [`super::Beta`] and is never
//! upgraded. Below 1.21.40 a document is not loaded.
//!
//! Every step compares the document's own `format_version`, not the struct's: a 1.21.40
//! document gets the components each later format expected its author to write. An inserted
//! component is reported as `SynthesizedComponent`.

pub mod v1_21_100;
pub mod v1_21_110;
pub mod v1_21_120;
pub mod v1_21_40;
pub mod v1_21_70;
pub mod v1_21_90;

use super::component::ClientBiomeComponent;
use super::parse::PAYLOAD_KEY;
use super::{Beta, ClientBiome};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, JsonPath, Location, Phase, Severity,
};
use bedrock_addon_core::document::{
    Component, ComponentMap, ComponentRef, ComponentSet, Document, Rejected,
};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::versioning::UpgradeCx;

use v1_21_40::Component as OlderComponent;

/// The client-biome family of the version-chain machinery (hidden: only the loaders name it).
#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct ClientBiomeFamily;

bedrock_addon_core::version_chain! {
    family = ClientBiomeFamily;
    /// A client-biome document parsed by the struct its `format_version` selects, not upgraded.
    #[derive(Clone, Debug, PartialEq)]
    pub enum Any {
        /// Format 1.21.40: the first format, read from the whole document.
        V1_21_40(v1_21_40::ClientBiome),
        /// Format 1.21.70.
        V1_21_70(v1_21_70::ClientBiome),
        /// Format 1.21.90.
        V1_21_90(v1_21_90::ClientBiome),
        /// Format 1.21.100.
        V1_21_100(v1_21_100::ClientBiome),
        /// Format 1.21.110.
        V1_21_110(v1_21_110::ClientBiome),
        /// Format 1.21.120: the last format read from the whole document.
        V1_21_120(v1_21_120::ClientBiome),
        /// Format 1.21.130 and later: the newest document.
        V1_21_130(ClientBiome),
    }
    beta = /// `"format_version": "beta"`.
    Beta(Beta);
    molang = none;
}

/// Runs the remaining upgrade steps of a document [`ClientBiome::parse_as_written`] gave: the
/// same result [`ClientBiome::parse`](bedrock_addon_core::document::Content::parse) gives for its file. A
/// beta document is rejected here (it is a [`Beta`]).
pub fn upgrade(
    document: Document<Any>,
    cx: &LoadContext,
) -> Result<Document<ClientBiome>, Rejected> {
    super::load::upgrade_document(document, cx)
}

/// One struct of a format below 1.21.130: the whole document, with the components of `$set`.
macro_rules! legacy_document {
    ($(#[$doc:meta])* version = ($major:literal, $minor:literal, $patch:literal), set = $set:ty) => {
        $(#[$doc])*
        #[derive(Clone, Debug, PartialEq)]
        pub struct ClientBiome {
            /// The document's `format_version`, as written (JSON `format_version`, a string).
            pub format_version: String,
            /// The biome's name (JSON `minecraft:client_biome/description`).
            pub description: $crate::Description,
            /// The components (JSON `minecraft:client_biome/components`).
            pub components: ::bedrock_addon_core::document::ComponentMap<$set>,
            /// Unknown members of `minecraft:client_biome`.
            pub extra: ::bedrock_addon_core::document::Extra,
            /// Unknown members of the document's root.
            pub root_extra: ::bedrock_addon_core::document::Extra,
        }

        impl ::bedrock_addon_core::versioning::Versioned for ClientBiome {
            const VERSION: ::bedrock_addon_core::version::FormatVersion =
                ::bedrock_addon_core::version::FormatVersion::new($major, $minor, $patch);
        }

        impl ::bedrock_addon_core::versioning::VersionStruct for ClientBiome {
            fn parse(
                document: ::bedrock_addon_core::json::JsonSlice<'_>,
                cx: &::bedrock_addon_core::versioning::ParseCx<'_>,
                d: &mut ::bedrock_addon_core::diagnostic::Diagnostics,
            ) -> Option<Self> {
                use $crate::parse::{strict, whole};
                use ::bedrock_addon_core::versioning::Versioned;
                let read = strict(cx, d, |d| whole::<$set>(document, Self::VERSION, cx, d))?;
                Some(Self {
                    format_version: read.format_version,
                    description: read.payload.description,
                    components: read.payload.components,
                    extra: read.payload.extra,
                    root_extra: read.root_extra,
                })
            }

            fn write(&self) -> Result<::bedrock_addon_core::json::Value, ::bedrock_addon_core::json::EncodeError> {
                use $crate::write::{document, object};
                let payload = object(&self.description, &self.components, &self.extra)?;
                Ok(document(
                    &self.format_version,
                    payload.into(),
                    &self.root_extra,
                ))
            }
        }
    };
}
pub(crate) use legacy_document;

/// The path of the component `name` in a client-biome file.
fn component_path(name: &str) -> JsonPath {
    let mut path = JsonPath::new();
    path.push(PAYLOAD_KEY);
    path.push("components");
    path.push(name);
    path
}

/// An info finding of a step about the component `name` (vanilla is silent).
fn note(cx: &mut UpgradeCx<'_>, code: DiagnosticCode, at: JsonPath) {
    cx.diagnostics.push(Diagnostic::new(
        code,
        Severity::Info,
        Phase::Upgrade,
        Location::new(None, None, at, None),
    ));
}

/// The key of a map entry.
fn entry_name<'a, C>(entry: &ComponentRef<'a, C>) -> &'a str {
    match *entry {
        ComponentRef::Typed { name, .. }
        | ComponentRef::Dynamic { name, .. }
        | ComponentRef::Unknown { name, .. } => name,
        ComponentRef::Custom(custom) => &custom.name,
        _ => "",
    }
}

/// Whether `map` has an entry keyed `name`.
pub(crate) fn has<C: ComponentSet>(map: &ComponentMap<C>, name: &str) -> bool {
    map.iter().any(|entry| entry_name(&entry) == name)
}

/// Inserts a component a step synthesizes, and says so.
pub(crate) fn insert<C, T>(map: &mut ComponentMap<C>, component: T, cx: &mut UpgradeCx<'_>)
where
    C: ComponentSet + From<ClientBiomeComponent>,
    T: Component<Set = ClientBiomeComponent>,
{
    map.insert_typed(C::from(component.into_set()), true);
    note(
        cx,
        DiagnosticCode::SynthesizedComponent,
        component_path(T::NAME),
    );
}

/// The component `T` of an older map, with whether a step inserted it.
pub(crate) fn find<T>(map: &ComponentMap<OlderComponent>) -> Option<(T, bool)>
where
    T: Component<Set = ClientBiomeComponent> + Clone,
{
    map.iter().find_map(|entry| match entry {
        ComponentRef::Typed {
            value: OlderComponent::Newest(component),
            synthesized,
            ..
        } => T::from_set(component).map(|found| (found.clone(), synthesized)),
        _ => None,
    })
}

/// Writes back a component a step changed, in its place, and says so.
pub(crate) fn change<T>(
    map: &mut ComponentMap<OlderComponent>,
    component: T,
    synthesized: bool,
    cx: &mut UpgradeCx<'_>,
) where
    T: Component<Set = ClientBiomeComponent>,
{
    map.insert_typed(OlderComponent::Newest(component.into_set()), synthesized);
    note(
        cx,
        DiagnosticCode::SynthesizedComponent,
        component_path(T::NAME),
    );
}

/// The description's identifier path.
pub(crate) fn identifier_path() -> JsonPath {
    let mut path = JsonPath::new();
    path.push(PAYLOAD_KEY);
    path.push("description");
    path.push("identifier");
    path
}

/// An info finding about the document's identifier.
pub(crate) fn note_identifier(cx: &mut UpgradeCx<'_>, code: DiagnosticCode) {
    note(cx, code, identifier_path());
}

/// An info finding about the component `name`.
pub(crate) fn note_component(cx: &mut UpgradeCx<'_>, code: DiagnosticCode, name: &str) {
    note(cx, code, component_path(name));
}
