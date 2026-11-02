//! `StackLoader` and `StackSet::load_all` through the public API: a
//! repository's stacks loaded with `Transient` and `All` residency, the brarchive layer, the
//! report. Collection parsing itself is tested in-crate over toy collections (`load::tests`)
//! until the families add accessor lines.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::load::{
    LoadOptions, Loader, RepositoryLoader, Residency, RootId, Source, StackLoader, StackOptions,
    Stage,
};
use bedrock_addon::pack::{OpenOptions, PackOrigin, StackSet, WorldPacks};
use bedrock_addon::storage::PackPath;

use super::responder::{Order, Responder, manifest};

/// A world with behaviour pack 1 (a texture, a script, a brarchive of entities) and resource
/// pack 2 under the packs directory at root 1.
fn world(order: Order) -> Responder {
    let mut responder = Responder::new(order);
    // each is a minimal entity)
    let entity = |id: &str| {
        format!(
            r#"{{"format_version": "1.26.50", "minecraft:entity": {{"description": {{"identifier": "minecraft:{id}"}}, "components": {{}}}}}}"#
        )
        .into_bytes()
    };
    let (pig, cow) = (entity("pig"), entity("cow"));
    let archive = brarchive::serialize([("pig.json", &pig[..]), ("cow.json", &cow[..])]).unwrap();
    responder
        .file(1, "bp/manifest.json", manifest(1, "data"))
        .file(1, "bp/scripts/main.js", &b"export {}"[..])
        .file(1, "bp/__brarchive/entities.brarchive", archive)
        .file(1, "rp/manifest.json", manifest(2, "resources"))
        .file(1, "rp/textures/a.png", &b"png"[..]);
    responder
}

/// The world list naming both packs.
fn world_packs() -> WorldPacks {
    let entry = |n: u128| {
        format!(
            r#"[{{"pack_id": "{}", "version": [1, 0, 0]}}]"#,
            uuid::Uuid::from_u128(n)
        )
    };
    WorldPacks::parse(Some(entry(1).as_bytes()), Some(entry(2).as_bytes())).unwrap()
}

/// Repository, stacks and stack load, the last in `order`.
fn load(order: Order, options: StackOptions) -> (StackSet, Responder) {
    let (set, _, responder) = load_world(world(Order::Issue), &world_packs(), order, options);
    (set, responder)
}

/// The repository of `responder`'s tree, the stacks of `packs` and the stack load, the last
/// in `order`; with the responder of the repository load and the one of the stack load, each
/// with the log of its own requests.
fn load_world(
    mut responder: Responder,
    packs: &WorldPacks,
    order: Order,
    options: StackOptions,
) -> (StackSet, Responder, Responder) {
    let mut loader = RepositoryLoader::new(
        vec![Source::Directory {
            root: RootId(1),
            origin: PackOrigin::World,
        }],
        OpenOptions::default(),
    );
    let repository = responder.drive(&mut loader);
    let set = repository.stacks(packs, LoadOptions::default());
    let opened = responder;
    let mut responder = opened.reordered(order);
    let mut loader = StackLoader::new(set, options);
    let set = responder.drive(&mut loader);
    assert_eq!(loader.progress().stage, Stage::Finished);
    (set, opened, responder)
}

/// The files the loads read, sorted.
fn files_read(responders: [&Responder; 2]) -> Vec<String> {
    let mut read: Vec<String> = responders
        .iter()
        .flat_map(|responder| &responder.log.requests)
        .filter_map(|request| match request {
            bedrock_addon::load::Request::ReadFile { location, .. } => Some(location.to_string()),
            _ => None,
        })
        .collect();
    read.sort();
    read
}

/// The findings of `report` at a `pack_icon.png`: code, severity, the pack's name.
fn icon_findings(report: &bedrock_addon::pack::StackReport<'_>) -> Vec<(String, Severity, String)> {
    report
        .iter()
        .filter(|finding| {
            finding
                .location
                .path
                .as_ref()
                .is_some_and(|path| path.as_str() == "pack_icon.png")
        })
        .map(|finding| {
            (
                finding.code.name().to_owned(),
                finding.severity,
                finding
                    .location
                    .pack
                    .as_ref()
                    .map(|pack| pack.name().to_owned())
                    .unwrap_or_default(),
            )
        })
        .collect()
}

/// A load asks for no byte of an image, the pack icon included. So on
/// a stack of packs a loader opened, the icon lints say only what the listing says: a pack
/// without the file is `PackIconMissing`; a `pack_icon.png` that is no PNG is never read and
/// never judged. The lints that need the bytes run where the caller has them
/// (`Pack::check_icon`) or the pack was built with them resident.
#[test]
fn a_stack_load_requests_no_byte_of_a_pack_icon() {
    let not_a_png: &[u8] = b"GIF89a, not a PNG";
    let world_with_icon = || {
        let mut responder = world(Order::Issue);
        responder.file(1, "bp/pack_icon.png", not_a_png);
        responder
    };
    let (set, opened, loaded) = load_world(
        world_with_icon(),
        &world_packs(),
        Order::Issue,
        StackOptions::default(),
    );
    let report = set.load_all();
    let read = files_read([&opened, &loaded]);
    assert!(
        read.iter().any(|path| path == "#1/bp/manifest.json"),
        "the log holds the reads: {read:?}"
    );
    assert!(
        !read.iter().any(|path| path.contains("pack_icon")),
        "the icon was requested: {read:?}"
    );
    // Pack 1 has the file, which is not judged; pack 2 has none.
    assert_eq!(
        icon_findings(&report),
        [(
            "PackIconMissing".to_owned(),
            Severity::Info,
            "pack 2".to_owned()
        )]
    );
    assert!(report.at_least(Severity::Warning).next().is_none());
    // The caller that has the bytes judges them.
    let judged = bedrock_addon::pack::Pack::check_icon(not_a_png);
    assert_eq!(
        judged
            .iter()
            .map(|finding| (finding.code.name(), finding.severity))
            .collect::<Vec<_>>(),
        [("InvalidValue", Severity::Info)]
    );

    // With every file kept resident the icon is read as every other file is, for serving;
    // the lints are those of the packs the stack was built from, and say the same.
    let (set, opened, loaded) = load_world(
        world_with_icon(),
        &world_packs(),
        Order::Issue,
        StackOptions {
            behavior: Residency::All,
            resource: Residency::All,
            ..StackOptions::default()
        },
    );
    let read = files_read([&opened, &loaded]);
    assert_eq!(
        read.iter()
            .filter(|path| path.contains("pack_icon"))
            .count(),
        1,
        "{read:?}"
    );
    assert_eq!(
        icon_findings(&set.load_all()),
        [(
            "PackIconMissing".to_owned(),
            Severity::Info,
            "pack 2".to_owned()
        )]
    );
}

#[test]
fn a_transient_load_reads_the_brarchives_and_keeps_them() {
    let (set, responder) = load(Order::Issue, StackOptions::default());
    let reads: Vec<String> = responder
        .log
        .requests
        .iter()
        .map(|request| format!("{request:?}"))
        .collect();
    // accessor and not read: nothing of a pack without a script module runs)
    assert_eq!(reads.len(), 1, "{reads:?}");
    assert!(reads[0].contains("__brarchive/entities.brarchive"));
    let bp = set.behavior.packs().next().unwrap();
    assert_eq!(bp.name(), "pack 1");
    let report = set.load_all();
    assert!(!report.has_errors(), "{report}");
}

#[test]
fn residency_all_keeps_every_file_resident_for_chunk_serving() {
    let options = StackOptions {
        behavior: Residency::All,
        resource: Residency::All,
        ..StackOptions::default()
    };
    let (set, responder) = load(Order::Seeded(11), options);
    // The brarchive, then every other file not resident yet: the script and the texture.
    let mut read: Vec<String> = responder
        .log
        .requests
        .iter()
        .map(|request| match request {
            bedrock_addon::load::Request::ReadFile { location, .. } => location.to_string(),
            other => panic!("{other:?}"),
        })
        .collect();
    read.sort();
    assert_eq!(
        read,
        [
            "#1/bp/__brarchive/entities.brarchive",
            "#1/bp/scripts/main.js",
            "#1/rp/textures/a.png"
        ]
    );
    let debug = format!("{set:?}");
    assert!(
        debug.contains("pack 1") && debug.contains("pack 2"),
        "{debug}"
    );
}

#[test]
fn every_completion_order_gives_the_same_stacks_and_report() {
    let options = || StackOptions {
        behavior: Residency::All,
        ..StackOptions::default()
    };
    let (reference, _) = load(Order::Issue, options());
    let reference = (
        format!("{reference:?}"),
        format!("{:?}", reference.load_all()),
    );
    for order in Order::ALL {
        let (set, _) = load(order, options());
        let observed = (format!("{set:?}"), format!("{:?}", set.load_all()));
        assert_eq!(observed, reference, "{order:?}");
    }
}

#[test]
fn load_all_reports_the_pack_selection_once() {
    let (set, _) = load(Order::Reverse, StackOptions::default());
    let report = set.load_all();
    // The base-game layer of a host without vanilla packs: `PackNotFound` infos, once each.
    let selection: Vec<_> = set.selection().iter().collect();
    // bundle, which names the pack)
    let of_scripts = |finding: &&bedrock_addon::diagnostic::Diagnostic| {
        finding.code.name() == "ScriptFileNotLoadable"
    };
    let notes = report.iter().filter(of_scripts).count();
    assert_eq!(notes, usize::from(cfg!(feature = "script")));
    // After them each stack's own findings: here the icon lints of the packs, none of which
    // has a `pack_icon.png`, once per pack.
    let (icons, reported): (Vec<_>, Vec<_>) = report
        .iter()
        .filter(|finding| !of_scripts(finding))
        .partition(|finding| finding.code.name() == "PackIconMissing");
    assert_eq!(reported, selection);
    let stacked = set.behavior.packs().len() + set.resource.packs().len();
    assert!(stacked > 0);
    assert_eq!(icons.len(), stacked);
    assert!(icons.iter().all(|icon| icon.severity == Severity::Info));
    assert!(report.at_least(Severity::Warning).next().is_none());
    assert!(!report.has_errors());
    let lines = report.to_string();
    assert_eq!(lines.lines().count(), selection.len() + icons.len() + notes);
    // `by_pack` and `by_file` group only findings that name a pack or a file; the script note
    // names a pack that has an icon lint too.
    assert_eq!(report.by_pack().count(), stacked);
    assert!(
        report
            .by_file()
            .all(|(path, group)| !group.is_empty() && !path.is_root())
    );
    let _ = PackPath::ROOT;
}
