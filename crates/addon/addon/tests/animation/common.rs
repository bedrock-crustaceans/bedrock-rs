//! What the animation test binaries share: contexts per side and `min_engine_version`,
//! loading one file of either kind, finding diagnostics by code and vanilla text, round trips,
//! and in-memory pack stacks.

#![allow(dead_code, reason = "each test file uses part of the helpers")]

use std::path::PathBuf;
use std::sync::Arc;

use bedrock_addon::diagnostic::{Diagnostic, Diagnostics, Severity};
use bedrock_addon::document::{Content, Document, Rejected};
use bedrock_addon::families::animation::AnimationFile;
use bedrock_addon::families::animation_controller::AnimationControllerFile;
use bedrock_addon::load::{LoadContext, LoadOptions};
use bedrock_addon::pack::PackType;
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::MinEngineVersion;
use uuid::Uuid;

/// A context of a pack of `pack_type` with `min_engine_version` `version` (`None`: absent), the
/// Molang compile check on when the feature is.
pub fn context(pack_type: PackType, version: Option<(u16, u16, u16)>) -> LoadContext {
    let mut cx = LoadContext::for_pack(
        &LoadOptions::default(),
        Uuid::from_u128(7),
        pack_type,
        version.map(|(a, b, c)| MinEngineVersion::new(a, b, c)),
        false,
    );
    cx.molang_version = bedrock_addon::molang::pack_molang_version(cx.min_engine_version.as_ref());
    #[cfg(feature = "molang")]
    {
        cx.molang = Some(Arc::new(bedrock_addon::molang::MolangCheck::new()));
    }
    let _ = Arc::<()>::new;
    cx
}

/// A behaviour pack at `min_engine_version` `a.b.c`.
pub fn bp(a: u16, b: u16, c: u16) -> LoadContext {
    context(PackType::Behavior, Some((a, b, c)))
}

/// A resource pack at `min_engine_version` `a.b.c`.
pub fn rp(a: u16, b: u16, c: u16) -> LoadContext {
    context(PackType::Resource, Some((a, b, c)))
}

/// Loads `text` as an animation file.
pub fn animation(text: &str, cx: &LoadContext) -> Result<Document<AnimationFile>, Rejected> {
    AnimationFile::parse(
        &PackPath::from_static("animations/x.json"),
        text.as_bytes(),
        cx,
    )
}

/// Loads `text` as a controller file.
pub fn controller(
    text: &str,
    cx: &LoadContext,
) -> Result<Document<AnimationControllerFile>, Rejected> {
    AnimationControllerFile::parse(
        &PackPath::from_static("animation_controllers/x.json"),
        text.as_bytes(),
        cx,
    )
}

/// Requires a load to succeed.
#[track_caller]
pub fn loaded<T: std::fmt::Debug>(result: Result<Document<T>, Rejected>) -> Document<T> {
    match result {
        Ok(document) => document,
        Err(rejected) => panic!("rejected: {}", lines(&rejected.diagnostics).join("\n")),
    }
}

/// Requires a load to fail; its findings.
#[track_caller]
pub fn rejected<T: std::fmt::Debug>(result: Result<Document<T>, Rejected>) -> Diagnostics {
    match result {
        Ok(document) => panic!("loaded: {:#?}", document.value()),
        Err(rejected) => rejected.diagnostics,
    }
}

/// The findings, one line each: severity, code name, path and text.
pub fn lines(d: &Diagnostics) -> Vec<String> {
    d.iter()
        .map(|f| {
            format!(
                "{:?} {} {} {}",
                f.severity,
                f.code.name(),
                f.location.json_path,
                f.code
            )
        })
        .collect()
}

/// The finding with code `name` and severity, if any.
pub fn find<'d>(d: &'d Diagnostics, name: &str, severity: Severity) -> Option<&'d Diagnostic> {
    d.iter()
        .find(|f| f.code.name() == name && f.severity == severity)
}

/// Requires a finding with code `name`, `severity` and vanilla's `text` exactly.
#[track_caller]
pub fn expect(d: &Diagnostics, name: &str, severity: Severity, text: &str) {
    assert!(
        d.iter()
            .any(|f| f.code.name() == name && f.severity == severity && f.code.to_string() == text),
        "no {severity:?} {name} \"{text}\" in\n{}",
        lines(d).join("\n")
    );
}

/// Requires a finding with code `name` and `severity` whose text starts with `prefix`.
#[track_caller]
pub fn expect_prefix(d: &Diagnostics, name: &str, severity: Severity, prefix: &str) {
    assert!(
        d.iter().any(|f| f.code.name() == name
            && f.severity == severity
            && f.code.to_string().starts_with(prefix)),
        "no {severity:?} {name} \"{prefix}…\" in\n{}",
        lines(d).join("\n")
    );
}

/// Requires no finding at warning level or above.
#[track_caller]
pub fn clean(d: &Diagnostics) {
    let found: Vec<_> = d.at_least(Severity::Warning).collect();
    assert!(found.is_empty(), "{}", lines(d).join("\n"));
}

/// Writes `document`, reads the text again with `parse` and requires an equal value and an
/// identical second write.
#[track_caller]
pub fn round_trip<T: Content + PartialEq + std::fmt::Debug>(
    document: &Document<T>,
    parse: impl Fn(&str) -> Result<Document<T>, Rejected>,
) {
    let text = document.to_json().unwrap();
    let again = loaded(parse(&text));
    assert_eq!(again.value(), document.value(), "{text}");
    assert_eq!(again.to_json().unwrap(), text);
}

/// A stack set read from in-memory packs: `behavior` and `resource` packs, each a list of
/// `(path, text)` files, highest priority first; each pack gets a manifest at
/// `min_engine_version` 1.21.0.
pub fn stacks(
    behavior: &[&[(&str, &str)]],
    resource: &[&[(&str, &str)]],
) -> bedrock_addon::pack::StackSet {
    use crate::responder::{Order, Responder, manifest};
    use bedrock_addon::load::{RepositoryLoader, RootId, Source, StackLoader, StackOptions};
    use bedrock_addon::pack::{OpenOptions, PackOrigin, WorldPacks};

    let mut responder = Responder::new(Order::Issue);
    let mut n = 0_u128;
    let mut lists = [Vec::new(), Vec::new()];
    for (side, (packs, module)) in [(behavior, "data"), (resource, "resources")]
        .into_iter()
        .enumerate()
    {
        for files in packs {
            n += 1;
            // `("manifest.json", text)` replaces the generated manifest; `("@subpack", name)`
            // activates that subpack in the world's pack list.
            if !files.iter().any(|(path, _)| *path == "manifest.json") {
                responder.file(1, &format!("p{n}/manifest.json"), manifest(n, module));
            }
            let mut subpack = String::new();
            for (path, text) in *files {
                if *path == "@subpack" {
                    subpack = format!(r#", "subpack": "{text}""#);
                    continue;
                }
                responder.file(1, &format!("p{n}/{path}"), (*text).to_string());
            }
            lists[side].push(format!(
                r#"{{"pack_id": "{}", "version": [1, 0, 0]{subpack}}}"#,
                Uuid::from_u128(n)
            ));
        }
    }
    let mut repository = RepositoryLoader::new(
        vec![Source::Directory {
            root: RootId(1),
            origin: PackOrigin::World,
        }],
        OpenOptions::default(),
    );
    let repository = responder.drive(&mut repository);
    let behavior = format!("[{}]", lists[0].join(", "));
    let resource = format!("[{}]", lists[1].join(", "));
    let world = WorldPacks::parse(Some(behavior.as_bytes()), Some(resource.as_bytes())).unwrap();
    let set = repository.stacks(&world, LoadOptions::default());
    let mut loader = StackLoader::new(set, StackOptions::default());
    responder.drive(&mut loader)
}

/// The fixture directory of `family` (`animation/bp`, `client_animation`, …).
pub fn fixtures(family: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/animation")
        .join(family)
}

/// A fixture file's text.
pub fn fixture(family: &str, name: &str) -> String {
    std::fs::read_to_string(fixtures(family).join(name))
        .unwrap_or_else(|e| panic!("{family}/{name}: {e}"))
}
