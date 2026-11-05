//! Loading animation controller files: `Content` / `Definition`, the walk of
//! `animation_controllers/` and the combine rule (per identifier, the higher pack's definition
//! replaces the lower one whole).

use super::history::Any;
use super::parse::{self, V1_8_0, V1_10_0};
use super::{AnimationController, AnimationControllerFile};
use bedrock_addon_core::diagnostic::{DiagnosticCode, Diagnostics, Severity};
use bedrock_addon_core::document::{
    Content, Definition, Document, JsonContent, NoBeta, Notice, Origin, Placed, Registry, Rejected,
    VersionInfo,
};
use bedrock_addon_core::id::kind;
use bedrock_addon_core::json::{EncodeError, WriteStyle};
use bedrock_addon_core::load::{CollectionLoader, LoadContext, StackCx, Walk, WalkFile};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::known;
use bedrock_addon_core::version::{FormatVersion, SemVersion};
use bedrock_addon_support::animation::load::{
    FileContribution, combine, file_path, side_of, split, version_info, walk_origin, whole,
};

impl bedrock_addon_core::__private::Sealed for AnimationControllerFile {}
impl bedrock_addon_core::__private::Sealed for AnimationController {}

fn controller_parsed_as(written: &SemVersion) -> (FormatVersion, bool) {
    if *written < V1_10_0 {
        (known::V1_8_0, true)
    } else {
        (known::V1_10_0, false)
    }
}

impl Content for AnimationControllerFile {
    const NAME: &'static str = "animation controller file";
    const SIDES: &'static [PackSide] = &[PackSide::Behavior, PackSide::Resource];

    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        let origin = Origin::file(PackRef::app_data(), side_of(cx), path.clone());
        whole(parse::parse_file(bytes, cx), origin, controller_parsed_as)
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        bedrock_addon_core::json::write_json(&self.to_value()?, out, style)
    }
}

impl JsonContent for AnimationControllerFile {}

impl Definition for AnimationController {
    type Kind = kind::Animation;
    type File = AnimationControllerFile;
    type Beta = NoBeta<kind::Animation>;
}

impl Placed for AnimationControllerFile {
    /// `animation_controllers/<name>.json`, after the first controller
    /// (`controller.animation.<name>`).
    fn default_path(&self) -> PackPath {
        file_path(
            "animation_controllers",
            "controller.animation.",
            self.animation_controllers.keys().next(),
        )
    }
}

impl AnimationControllerFile {
    /// Parses one controller file without upgrading it: the struct its `format_version` selects;
    /// [`history::upgrade`](super::history::upgrade) finishes it.
    ///
    /// # Errors
    ///
    /// Returns [`Rejected`](bedrock_addon_core::document::Rejected) with the findings the game's content
    /// log would print when the bytes cannot be read as this file at all: not JSON, a
    /// `format_version` with no struct here, or a document that does not fit the struct it selects.
    pub fn parse_as_written(
        path: &PackPath,
        bytes: &[u8],
        cx: &LoadContext,
    ) -> Result<Document<Any>, Rejected> {
        let origin = Origin::file(PackRef::app_data(), side_of(cx), path.clone());
        let (any, written, diagnostics) = parse::parse_as_written(bytes, cx);
        let diagnostics = diagnostics.located(&origin);
        match (any, written) {
            (Some(any), Some(written)) => {
                let as_ = any.parsed_version();
                Ok(Document::new(
                    any,
                    origin,
                    version_info(&written, as_, false),
                    diagnostics,
                ))
            }
            _ => Err(Rejected::new(origin, diagnostics)),
        }
    }
}

/// `history::upgrade`: the step and the checks on a document `parse_as_written` gave.
pub(crate) fn upgrade_document(
    document: Document<Any>,
    cx: &LoadContext,
) -> Result<Document<AnimationControllerFile>, Rejected> {
    let origin = document.origin().clone();
    let version = document.version().clone();
    let mut diagnostics = Diagnostics::default();
    let written = version
        .written
        .map_or(V1_8_0, |v| SemVersion::new(v.major, v.minor, v.patch));
    let upgraded = matches!(document.value(), Any::V1_8_0(_));
    let file = parse::upgrade(document.into_value(), &written, cx, &mut diagnostics)
        .and_then(|file| parse::finish(file, &written, cx, &mut diagnostics));
    let diagnostics = diagnostics.located(&origin);
    let info = VersionInfo::new(
        version.written,
        false,
        version.effective,
        version.parsed_as,
        upgraded,
    );
    match file {
        Some(file) => Ok(Document::new(file, origin, info, diagnostics)),
        None => Err(Rejected::new(origin, diagnostics)),
    }
}

fn parse_controller_file(
    file: &WalkFile,
    cx: &LoadContext,
) -> FileContribution<AnimationController> {
    let result = whole(
        parse::parse_file(&file.bytes, cx),
        walk_origin(file),
        controller_parsed_as,
    );
    split(
        result,
        "animation_controllers",
        |f: AnimationControllerFile| f.animation_controllers.into_iter().collect(),
    )
    .with_file(file)
}

/// `animation_controllers/*.json`, packs bottom-up, sub-folders not read.
const CONTROLLER_WALK: Walk = Walk::json_children_in(&["animation_controllers"]);

fn combine_controllers(
    parsed: Vec<FileContribution<AnimationController>>,
    stack: &StackCx<'_>,
) -> Registry<AnimationController> {
    combine(
        parsed,
        stack,
        Notice {
            code: DiagnosticCode::AnimationControllerShadowed,
            severity: Severity::Info,
        },
    )
}

/// The loader of `animation_controllers()` on the stack of `side`:
/// `animation_controllers/*.json`, combined per identifier, the higher pack winning.
pub fn load_controllers(
    side: PackSide,
) -> CollectionLoader<Registry<AnimationController>, FileContribution<AnimationController>> {
    let _ = side;
    CollectionLoader {
        walk: CONTROLLER_WALK,
        parse: parse_controller_file,
        combine: combine_controllers,
        needs: &[],
        then: None,
    }
}
