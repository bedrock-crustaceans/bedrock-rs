//! Loading animation files: `Content` / `Definition`, the walk of `animations/` and the combine
//! rule (per identifier, the higher pack's definition replaces the lower one whole).

use super::parse;
use super::{Animation, AnimationFile};
use bedrock_addon_core::document::{
    Content, Definition, Document, NoBeta, Notice, Origin, Placed, Registry, Rejected,
};
use bedrock_addon_core::id::kind;
use bedrock_addon_core::json::{EncodeError, WriteStyle};
use bedrock_addon_core::load::{
    AppDataPlace, CollectionLoader, ListDepth, LoadContext, StackCx, Walk, WalkFile, WalkOrder,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_support::animation::load::{
    FileContribution, combine, file_path, is_json, side_of, split, walk_origin, whole,
};

impl bedrock_addon_core::__private::Sealed for AnimationFile {}
impl bedrock_addon_core::__private::Sealed for Animation {}

impl Content for AnimationFile {
    const NAME: &'static str = "animation file";
    const SIDES: &'static [PackSide] = &[PackSide::Behavior, PackSide::Resource];

    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        let origin = Origin::new(
            PackRef::app_data(),
            side_of(cx),
            path.clone(),
            None,
            Vec::new(),
        );
        whole(parse::parse_file(bytes, cx), origin, |_| {
            (FormatVersion::new(1, 8, 0), false)
        })
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        bedrock_addon_core::json::write_json(&self.to_value()?, out, style)
    }

    fn to_json(&self) -> Result<String, EncodeError> {
        let mut out = Vec::new();
        self.write_json(&mut out, WriteStyle::default())?;
        Ok(String::from_utf8(out).unwrap_or_default())
    }
}

impl Definition for Animation {
    type Kind = kind::Animation;
    type File = AnimationFile;
    type Beta = NoBeta<kind::Animation>;
}

impl Placed for AnimationFile {
    /// `animations/<name>.json`, after the first animation (`animation.<name>`).
    fn default_path(&self) -> PackPath {
        file_path("animations", "animation.", self.animations.keys().next())
    }
}

fn parse_animation_file(file: &WalkFile, cx: &LoadContext) -> FileContribution<Animation> {
    let result = whole(
        parse::parse_file(&file.bytes, cx),
        walk_origin(file),
        |_| (FormatVersion::new(1, 8, 0), false),
    );
    split(result, "animations", |f: AnimationFile| {
        f.animations.into_iter().collect()
    })
    .with_file(file)
}

/// `animations/**/*.json`, packs bottom-up. Vanilla's directory strategy recurses for every pack
/// but those shipped inside the app, which it lists flat.
const ANIMATION_WALK: Walk = Walk {
    directories: &["animations"],
    depth: ListDepth::Recursive,
    accept: is_json,
    order: WalkOrder::BottomUp,
    directory_major: false,
    app_data: &[],
    app_data_place: AppDataPlace::After,
    in_pack: None,
    shadowed: None,
    read_error: None,
};

fn combine_animations(
    parsed: Vec<FileContribution<Animation>>,
    stack: &StackCx<'_>,
) -> Registry<Animation> {
    combine(parsed, stack, Notice::shadowed())
}

/// The loader of `animations()` on the stack of `side`: `animations/**/*.json`, combined per
/// identifier, the higher pack winning.
#[must_use]
pub fn load_animations(
    side: PackSide,
) -> CollectionLoader<Registry<Animation>, FileContribution<Animation>> {
    let _ = side;
    CollectionLoader {
        walk: ANIMATION_WALK,
        parse: parse_animation_file,
        combine: combine_animations,
        needs: &[],
        then: None,
    }
}
