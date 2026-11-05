//! The side files of the UI images of a stack: `textures/ui/**/*.json`, each the highest
//! pack's copy whichever pack supplies the image. The file's own reader is `UiTextureInfo`'s;
//! this is what the stack makes of the copies.

use std::collections::HashSet;

use bedrock_addon_core::load::{
    Accept, AppDataPlace, CollectionLoader, ListDepth, LoadContext, StackCx, Walk, WalkFile,
    WalkOrder,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef};
use indexmap::IndexMap;

use super::UiTextureInfo;
use bedrock_addon_core::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon_core::document::{Content, Document, Files, FilesBuilder, FilesRule, Rejected};
use bedrock_addon_support::fixed_path::{below, document, file_finding, locate, origin};

/// The directory whose images have side files.
const DIRECTORY: &str = "textures/ui";
/// The extensions an image is looked up with, in order; the first is the path as written.
const IMAGE: [&str; 6] = [".tga", ".png", ".jpg", ".jpeg", ".hdr", ".ktx"];

/// One copy of a side file: the path it is filed under and the document or its rejection.
pub type SideFileCopy = (PackPath, Result<Document<UiTextureInfo>, Rejected>);

/// The extension of a side file, as the game appends it to the image's path.
const JSON: &str = ".json";

fn is_json(path: &PackPath) -> bool {
    path.as_str().strip_suffix(JSON).is_some()
}

/// One file's copy, read by the type's own reader and placed in its pack.
fn parse(file: &WalkFile, cx: &LoadContext) -> SideFileCopy {
    let origin = origin(file.pack.clone(), file.side, &file.path);
    let copy = match UiTextureInfo::parse(&file.path, &file.bytes, cx) {
        Ok(read) => {
            let diagnostics = read.diagnostics().clone();
            Ok(document(read.into_value(), origin, diagnostics))
        }
        Err(rejected) => {
            let mut diagnostics = rejected.diagnostics;
            locate(&mut diagnostics, &origin);
            Err(Rejected::new(origin, diagnostics))
        }
    };
    (file.key.clone(), copy)
}

/// The `Combine` job of `rp.ui_texture_infos()`: the highest pack's copy of each path; a
/// highest copy that is refused leaves the image without a side file.
/// The game loads the first pack's copy from the top that has the JSON and does not look at
/// the parse result, so no lower copy stands in.
fn combine(parsed: Vec<SideFileCopy>, cx: &StackCx<'_>) -> Files<UiTextureInfo> {
    // The images of the stack, each as the path its side file is named after: a file below
    // `textures/ui/` without its image extension.
    let mut images: HashSet<&str> = HashSet::new();
    for pack in cx.packs {
        for file in below(pack, DIRECTORY) {
            let key = file.key.as_str();
            if key.strip_suffix(JSON).is_none() {
                let stem = IMAGE
                    .iter()
                    .find_map(|extension| key.strip_suffix(extension));
                images.insert(stem.unwrap_or(key));
            }
        }
    }
    let mut builder = FilesBuilder::new(FilesRule::TopWins, cx.options);
    // Each path with where its highest copy lies (the walk is lowest pack first).
    let mut highest: IndexMap<PackPath, (PackRef, PackPath)> = IndexMap::new();
    for (key, copy) in parsed {
        let origin = match &copy {
            Ok(read) => read.origin(),
            Err(rejected) => &rejected.origin,
        };
        highest.insert(key.clone(), (origin.pack.clone(), origin.path.clone()));
        builder.push(key, copy);
    }
    // No pack has an image the side file is for; the game never asks for it. Lowest pack
    // first, by path.
    let mut orphans: Vec<(&PackPath, &(PackRef, PackPath))> = highest
        .iter()
        .filter(|(key, _)| {
            let stem = key.as_str().strip_suffix(JSON).unwrap_or(key.as_str());
            !images.contains(stem)
        })
        .collect();
    orphans.sort_by(|a, b| {
        let rank = |pack: &PackRef| std::cmp::Reverse(pack.index());
        (rank(&a.1.0), a.0.as_str()).cmp(&(rank(&b.1.0), b.0.as_str()))
    });
    for (_, (pack, path)) in orphans {
        builder.push_diagnostic(file_finding(
            DiagnosticCode::IgnoredFile { text: None },
            Severity::Info,
            pack,
            path,
        ));
    }
    builder.finish()
}

/// The loader of `rp.ui_texture_infos()`: every `.json` file below `textures/ui/` of every
/// resource pack. The images are known from each pack's listing; none is read.
pub const fn load_texture_infos() -> CollectionLoader<Files<UiTextureInfo>, SideFileCopy> {
    CollectionLoader {
        walk: Walk {
            directories: &[DIRECTORY],
            depth: ListDepth::Recursive,
            accept: Accept::Fn(is_json),
            order: WalkOrder::BottomUp,
            directory_major: false,
            app_data: &[],
            app_data_place: AppDataPlace::After,
            in_pack: None,
            shadowed: None,
            read_error: None,
        },
        parse,
        combine,
        needs: &[],
        then: None,
    }
}

#[cfg(test)]
mod tests {
    use bedrock_addon_core::load::LoadOptions;

    use super::*;
    use bedrock_addon_core::diagnostic::Diagnostics;
    use bedrock_addon_core::test_cost::assert_linear_at;
    use bedrock_addon_support::fixed_path::lone;

    /// The highest copy of each path is chosen, and the side files without an image named, in
    /// time linear in the listing (the order of the findings aside): two copies of every path,
    /// no image.
    #[test]
    fn the_winners_of_a_listing_are_chosen_in_linear_time() {
        let cx = LoadContext::detached();
        let build = |entries: usize| -> Vec<(PackPath, Document<UiTextureInfo>)> {
            (0..entries)
                .map(|i| {
                    let path =
                        PackPath::new(&format!("textures/ui/i{}.json", i / 2)).expect("a path");
                    let read = UiTextureInfo::parse(&path, br#"{ "nineslice_size": 2 }"#, &cx)
                        .expect("a side file");
                    let copy = document(read.into_value(), lone(&path), Diagnostics::default());
                    (path, copy)
                })
                .collect()
        };
        let options = LoadOptions::default();
        assert_linear_at("side_files::combine", 25_000, build, |copies| {
            let parsed = copies
                .iter()
                .cloned()
                .map(|(key, copy)| (key, Ok(copy)))
                .collect();
            let files = combine(parsed, &StackCx::new(&options, &[]));
            assert_eq!(files.iter().count(), copies.len() / 2);
            assert_eq!(files.diagnostics().count(), copies.len() / 2);
        });
    }
}
