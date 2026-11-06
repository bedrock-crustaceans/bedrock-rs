//! The geometry file as written in older format versions, and the steps that upgrade it to
//! [`super::GeometryFile`].
//!
//! The chain is `v0_0_0 → v1_8_0 → v1_12_0 → v1_14_0 → v1_16_0 → v1_19_30 → v1_21_0 →
//! v1_26_40`, and a file parses as the struct whose version is the highest one not above its
//! `format_version`. There is no lowest version (a file without a readable `format_version`
//! is 1.0.0) and no beta form; a file above the engine version is not loaded.
//!
//! The two shapes below 1.12.0 keep their geometries as `"geometry.<name>"` members of the
//! root and have member types of their own. From 1.12.0 the shape is the newest one: each
//! era's struct is its file type, whose reader declares only the members of its schema, and the
//! member types are the newest ones (a member a schema does not declare is absent).

pub mod v0_0_0;
pub mod v1_12_0;
pub mod v1_14_0;
pub mod v1_16_0;
pub mod v1_19_30;
pub mod v1_21_0;
pub mod v1_8_0;

use super::GeometryFile;
use super::read::{Era, read_file};
use super::walk::rules;
use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::document::{Document, Rejected};
use bedrock_addon_core::json::node_tree::{Node, Walk};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value, WriteStyle};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
use bedrock_addon_core::version::{FormatVersion, known};
use bedrock_addon_core::versioning::{ParseCx, Upgrade, UpgradeCx, VersionStruct, Versioned};

/// The newest struct under its chain name: format 1.26.40 and later.
pub mod v1_26_40 {
    pub use super::super::GeometryFile;
}

/// The versions of the chain's structs, oldest first.
pub const VERSIONS: [FormatVersion; 8] = [
    known::V0_0_0,
    known::V1_8_0,
    FormatVersion::new(1, 12, 0),
    FormatVersion::new(1, 14, 0),
    FormatVersion::new(1, 16, 0),
    FormatVersion::new(1, 19, 30),
    FormatVersion::new(1, 21, 0),
    known::V1_26_40,
];

/// Below this version an undeclared or wrong-typed member is removed with a warning; from it,
/// such a member fails the file.
pub const STRICT_FROM: FormatVersion = FormatVersion::new(1, 19, 30);

/// The walk of a struct's parse: lenient below 1.19.30.
pub(crate) fn walk_for(cx: &ParseCx<'_>) -> Walk {
    Walk::new(rules(cx.original_version < STRICT_FROM), &cx.at)
}

/// Hands a walk's findings over; `value` when the walk raised no error.
pub(crate) fn settle<T>(walk: Walk, value: Option<T>, d: &mut Diagnostics) -> Option<T> {
    let failed = walk.diagnostics.has_errors();
    d.extend(walk.diagnostics);
    value.filter(|_| !failed)
}

/// Reads a file of 1.12.0 or later with the schema of the struct `schema`.
pub(crate) fn parse_modern(
    payload: JsonSlice<'_>,
    schema: FormatVersion,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
) -> Option<(Option<bool>, Vec<super::Geometry>)> {
    let mut walk = walk_for(cx);
    let era = Era::of(schema, cx.load.engine_version);
    let file = Node::of(payload).and_then(|root| read_file(&mut walk, root, era));
    settle(walk, file, d).map(|file| (file.debug, file.geometries))
}

/// Declares the file struct of an era from 1.12.0 on, with its chain traits.
macro_rules! modern_file {
    ($version:expr; $(#[$doc:meta])*) => {
        $(#[$doc])*
        #[derive(Clone, Debug, PartialEq)]
        pub struct GeometryFile {
            /// `format_version`, as written.
            pub format_version: ::bedrock_addon_core::version::FormatVersion,
            /// `debug`.
            pub debug: Option<bool>,
            /// `minecraft:geometry`, in file order.
            pub geometries: Vec<$crate::Geometry>,
        }

        impl ::bedrock_addon_core::versioning::Versioned for GeometryFile {
            const VERSION: ::bedrock_addon_core::version::FormatVersion = $version;
        }

        impl ::bedrock_addon_core::molang::MolangFields for GeometryFile {
            fn visit_molang(&mut self, visitor: &mut ::bedrock_addon_core::molang::MolangVisitor<'_>) {
                $crate::molang::visit(&mut self.geometries, visitor);
            }
        }

        impl ::bedrock_addon_core::versioning::VersionStruct for GeometryFile {
            fn parse(
                payload: ::bedrock_addon_core::json::JsonSlice<'_>,
                cx: &::bedrock_addon_core::versioning::ParseCx<'_>,
                d: &mut ::bedrock_addon_core::diagnostic::Diagnostics,
            ) -> Option<Self> {
                let (debug, geometries) =
                    $crate::history::parse_modern(payload, $version, cx, d)?;
                Some(Self {
                    format_version: cx.original_version,
                    debug,
                    geometries,
                })
            }

            fn write(&self) -> Result<::bedrock_addon_core::json::Value, ::bedrock_addon_core::json::EncodeError> {
                $crate::write::modern(self.format_version, self.debug, &self.geometries)
            }
        }
    };
}
pub(crate) use modern_file;

impl Versioned for GeometryFile {
    const VERSION: FormatVersion = known::V1_26_40;
}

impl VersionStruct for GeometryFile {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let (debug, geometries) = parse_modern(payload, Self::VERSION, cx, d)?;
        Some(Self {
            format_version: cx.original_version,
            debug,
            geometries,
        })
    }

    fn write(&self) -> Result<Value, EncodeError> {
        self.to_value()
    }
}

/// A geometry file parsed by the struct its `format_version` selects, not upgraded.
// Hand-written: waits on the `Family` skeleton (not yet adopted by this family).
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum GeometryHistory {
    /// Below 1.8.0.
    V0_0_0(v0_0_0::GeometryFile),
    /// 1.8.0 up to 1.12.0.
    V1_8_0(v1_8_0::GeometryFile),
    /// 1.12.0 up to 1.14.0.
    V1_12_0(v1_12_0::GeometryFile),
    /// 1.14.0 up to 1.16.0.
    V1_14_0(v1_14_0::GeometryFile),
    /// 1.16.0 up to 1.19.30.
    V1_16_0(v1_16_0::GeometryFile),
    /// 1.19.30 up to 1.21.0.
    V1_19_30(v1_19_30::GeometryFile),
    /// 1.21.0 up to 1.26.40.
    V1_21_0(v1_21_0::GeometryFile),
    /// 1.26.40 and later: the newest document.
    V1_26_40(GeometryFile),
}

impl GeometryHistory {
    /// The index in [`VERSIONS`] of the struct that parses a file of `version`.
    pub(crate) fn index_of(version: FormatVersion) -> usize {
        VERSIONS
            .iter()
            .rposition(|start| *start <= version)
            .unwrap_or(0)
    }

    /// Parses `payload` (the whole file) with the struct of `cx.original_version`.
    pub(crate) fn parse(
        payload: JsonSlice<'_>,
        cx: &ParseCx<'_>,
        d: &mut Diagnostics,
    ) -> Option<Self> {
        Some(match Self::index_of(cx.original_version) {
            0 => Self::V0_0_0(VersionStruct::parse(payload, cx, d)?),
            1 => Self::V1_8_0(VersionStruct::parse(payload, cx, d)?),
            2 => Self::V1_12_0(VersionStruct::parse(payload, cx, d)?),
            3 => Self::V1_14_0(VersionStruct::parse(payload, cx, d)?),
            4 => Self::V1_16_0(VersionStruct::parse(payload, cx, d)?),
            5 => Self::V1_19_30(VersionStruct::parse(payload, cx, d)?),
            6 => Self::V1_21_0(VersionStruct::parse(payload, cx, d)?),
            _ => Self::V1_26_40(VersionStruct::parse(payload, cx, d)?),
        })
    }

    /// Runs every remaining step.
    pub(crate) fn upgrade_to_end(self, cx: &mut UpgradeCx<'_>) -> GeometryFile {
        let mut any = self;
        loop {
            any = match any {
                Self::V0_0_0(file) => Self::V1_8_0(file.upgrade(cx)),
                Self::V1_8_0(file) => Self::V1_12_0(file.upgrade(cx)),
                Self::V1_12_0(file) => Self::V1_14_0(file.upgrade(cx)),
                Self::V1_14_0(file) => Self::V1_16_0(file.upgrade(cx)),
                Self::V1_16_0(file) => Self::V1_19_30(file.upgrade(cx)),
                Self::V1_19_30(file) => Self::V1_21_0(file.upgrade(cx)),
                Self::V1_21_0(file) => Self::V1_26_40(file.upgrade(cx)),
                Self::V1_26_40(file) => return file,
            };
        }
    }

    /// The version of the struct that parsed the file.
    pub fn parsed_version(&self) -> FormatVersion {
        VERSIONS[match self {
            Self::V0_0_0(_) => 0,
            Self::V1_8_0(_) => 1,
            Self::V1_12_0(_) => 2,
            Self::V1_14_0(_) => 3,
            Self::V1_16_0(_) => 4,
            Self::V1_19_30(_) => 5,
            Self::V1_21_0(_) => 6,
            Self::V1_26_40(_) => 7,
        }]
    }

    /// Writes the file in its own struct's shape (a round trip without upgrading).
    ///
    /// # Errors
    ///
    /// Returns an [`EncodeError`] when a value cannot be written as JSON.
    pub fn to_json(&self) -> Result<String, EncodeError> {
        let value = match self {
            Self::V0_0_0(file) => file.write(),
            Self::V1_8_0(file) => file.write(),
            Self::V1_12_0(file) => file.write(),
            Self::V1_14_0(file) => file.write(),
            Self::V1_16_0(file) => file.write(),
            Self::V1_19_30(file) => file.write(),
            Self::V1_21_0(file) => file.write(),
            Self::V1_26_40(file) => file.write(),
        }?;
        let mut out = Vec::new();
        bedrock_addon_core::json::write_json(&value, &mut out, WriteStyle::default())?;
        Ok(String::from_utf8(out).unwrap_or_default())
    }
}

impl MolangFields for GeometryHistory {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        match self {
            Self::V0_0_0(_) | Self::V1_8_0(_) => {}
            Self::V1_12_0(file) => file.visit_molang(visitor),
            Self::V1_14_0(file) => file.visit_molang(visitor),
            Self::V1_16_0(file) => file.visit_molang(visitor),
            Self::V1_19_30(file) => file.visit_molang(visitor),
            Self::V1_21_0(file) => file.visit_molang(visitor),
            Self::V1_26_40(file) => file.visit_molang(visitor),
        }
    }
}

/// Runs the remaining upgrade steps and the checks of a document
/// [`GeometryFile::parse_as_written`] gave: the same result [`load_file`](super::load_file)
/// gives for its file. It works on the document, which holds the origin, so the split lets a
/// caller inspect the file as written before it is upgraded.
///
/// # Errors
///
/// Returns [`Rejected`] when an upgrade step or a check refuses the document; it carries every
/// finding about the file.
pub fn upgrade(
    document: Document<GeometryHistory>,
    cx: &LoadContext,
) -> Result<Document<GeometryFile>, Rejected> {
    super::upgrade_document(document, cx)
}
