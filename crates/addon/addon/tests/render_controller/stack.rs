//! `rc22`–`rc26` (and `rc04` seen from
//! the registry), the accessor, and where a file is written.

use bedrock_addon::diagnostic::{Diagnostic, Severity};
use bedrock_addon::document::{Placed, Registry};
use bedrock_addon::families::render_controller::{RenderController, RenderControllerFile};
use bedrock_addon::id::RenderControllerId;

use super::common::{line, stacks};
use super::rows::{OK, file};

fn cow(geometry: &str) -> String {
    file(&format!(
        r#""controller.render.cow": {{"geometry": "{geometry}", "textures": ["Texture.default"]}}"#
    ))
}

fn id(name: &str) -> RenderControllerId {
    RenderControllerId::parse(name)
}

fn found<'r>(
    registry: &'r Registry<RenderController>,
    name: &str,
    severity: Severity,
) -> Vec<&'r Diagnostic> {
    registry
        .diagnostics()
        .filter(|f| f.code.name() == name && f.severity == severity)
        .collect()
}

fn all(registry: &Registry<RenderController>) -> Vec<String> {
    registry.diagnostics().map(line).collect()
}

#[test]
fn rc22_stack_override_the_highest_pack_wins_with_a_shadow_info_deviation() {
    let p0 = cow("Geometry.top");
    let p1 = cow("Geometry.bottom");
    let set = stacks(&[
        &[("render_controllers/cow.json", &p0)],
        &[("render_controllers/cow.json", &p1)],
    ]);
    let registry = set.resource.render_controllers();
    assert_eq!(registry.len(), 1);
    let cow = registry.get(&id("controller.render.cow")).unwrap();
    assert_eq!(cow.geometry.as_str(), "geometry.top");
    assert_eq!(cow.origin().pack.index(), Some(0));
    assert_eq!(cow.origin().key.as_deref(), Some("controller.render.cow"));
    // The name folds case like the file does.
    assert!(registry.get(&id("Controller.Render.COW")).is_some());
    let shadowed = found(registry, "RenderControllerShadowed", Severity::Info);
    assert_eq!(shadowed.len(), 1, "{:?}", all(registry));
    assert_eq!(
        shadowed[0]
            .location
            .pack
            .as_ref()
            .and_then(bedrock_addon::pack::PackRef::index),
        Some(1)
    );
    assert!(found(registry, "RenderControllerFallback", Severity::Info).is_empty());
}

#[test]
fn rc22_one_pack_alone_shadows_nothing() {
    let p0 = cow("Geometry.top");
    let set = stacks(&[&[("render_controllers/cow.json", &p0)]]);
    let registry = set.resource.render_controllers();
    assert!(all(registry).iter().all(|l| !l.contains("Shadowed")));
    assert_eq!(registry.len(), 1);
}

#[test]
fn rc23_stack_fallback_a_failed_override_keeps_the_lower_definition_deviation() {
    let p0 = file(r#""controller.render.cow": {"geometry": "Geometry.top"}"#);
    let p1 = cow("Geometry.bottom");
    let set = stacks(&[
        &[("render_controllers/cow.json", &p0)],
        &[("render_controllers/cow.json", &p1)],
    ]);
    let registry = set.resource.render_controllers();
    let cow = registry.get(&id("controller.render.cow")).unwrap();
    assert_eq!(cow.geometry.as_str(), "geometry.bottom");
    assert_eq!(cow.origin().pack.index(), Some(1));
    let errors = found(registry, "MissingRequiredField", Severity::Error);
    assert_eq!(errors.len(), 1, "{:?}", all(registry));
    assert_eq!(
        errors[0].code.to_string(),
        "Render controller must contain textures."
    );
    let fallback = found(registry, "RenderControllerFallback", Severity::Info);
    assert_eq!(fallback.len(), 1, "{:?}", all(registry));
    assert_eq!(
        fallback[0]
            .location
            .pack
            .as_ref()
            .and_then(bedrock_addon::pack::PackRef::index),
        Some(0)
    );
    assert!(found(registry, "RenderControllerShadowed", Severity::Info).is_empty());
    assert_eq!(registry.rejected().len(), 1);
}

#[test]
fn rc23_a_failed_controller_with_nothing_below_is_no_fallback() {
    let p0 = file(r#""controller.render.cow": {"geometry": "Geometry.top"}"#);
    let set = stacks(&[&[("render_controllers/cow.json", &p0)]]);
    let registry = set.resource.render_controllers();
    assert!(registry.is_empty());
    assert!(found(registry, "RenderControllerFallback", Severity::Info).is_empty());
}

#[test]
fn rc24_stack_file_failure_removes_nothing() {
    let p0 = cow("Geometry.top").replace("\"textures\"", "\"nope\": 1, \"textures\"");
    let p1 = cow("Geometry.bottom");
    let set = stacks(&[
        &[("render_controllers/cow.json", &p0)],
        &[("render_controllers/cow.json", &p1)],
    ]);
    let registry = set.resource.render_controllers();
    let cow = registry.get(&id("controller.render.cow")).unwrap();
    assert_eq!(cow.geometry.as_str(), "geometry.bottom");
    assert_eq!(
        found(registry, "UnrecognizedField", Severity::Error).len(),
        1
    );
    assert!(found(registry, "RenderControllerShadowed", Severity::Info).is_empty());
}

#[test]
fn rc04_no_textures_from_the_registry() {
    const TEXT: &str = "Render controller must contain textures.";
    let p0 = file(&format!(
        r#""controller.render.test": {{"geometry": "Geometry.default"}}, {OK}"#
    ));
    let set = stacks(&[&[("render_controllers/a.json", &p0)]]);
    let registry = set.resource.render_controllers();
    assert!(registry.get(&id("controller.render.test")).is_none());
    assert!(registry.get(&id("controller.render.ok")).is_some());
    assert_eq!(registry.rejected().len(), 1);
    assert_eq!(
        registry.rejected()[0].origin.key.as_deref(),
        Some("controller.render.test")
    );
    // The dropped controller's rejection holds its error; its sibling's document does not.
    let rejected: Vec<String> = registry.rejected()[0]
        .diagnostics
        .iter()
        .map(|f| f.code.to_string())
        .collect();
    assert_eq!(rejected, [TEXT]);
    let ok = registry.get(&id("controller.render.ok")).unwrap();
    assert!(ok.diagnostics().is_empty(), "{:?}", ok.diagnostics());
    // Reported once in the registry, at the dropped controller.
    let errors = found(registry, "MissingRequiredField", Severity::Error);
    assert_eq!(errors.len(), 1, "{:?}", all(registry));
    assert_eq!(
        errors[0].location.json_path.to_string(),
        "render_controllers/controller.render.test/textures"
    );
}

#[test]
fn rc04_two_dropped_controllers_each_hold_their_own_findings() {
    let p0 = file(&format!(
        r#""controller.render.a": {{"geometry": "Geometry.default"}}, "controller.render.b": {{"geometry": "Geometry.default"}}, {OK}"#
    ));
    let set = stacks(&[&[("render_controllers/a.json", &p0)]]);
    let registry = set.resource.render_controllers();
    let rejected: Vec<(Option<&str>, Vec<String>)> = registry
        .rejected()
        .iter()
        .map(|r| {
            (
                r.origin.key.as_deref(),
                r.diagnostics
                    .iter()
                    .map(|f| f.location.json_path.to_string())
                    .collect(),
            )
        })
        .collect();
    assert_eq!(
        rejected,
        [
            (
                Some("controller.render.a"),
                vec!["render_controllers/controller.render.a/textures".to_owned()]
            ),
            (
                Some("controller.render.b"),
                vec!["render_controllers/controller.render.b/textures".to_owned()]
            ),
        ]
    );
}

#[test]
fn rc26_file_selection_deviation() {
    let x = cow("Geometry.x");
    let y = file(OK);
    let set = stacks(&[&[
        ("render_controllers/x.JSON", &x),
        ("render_controllers/x.txt", &x),
        ("render_controllers/sub/y.json", &y),
    ]]);
    let registry = set.resource.render_controllers();
    assert_eq!(registry.len(), 1, "{:?}", all(registry));
    let ok = registry.get(&id("controller.render.ok")).unwrap();
    assert_eq!(ok.origin().path.as_str(), "render_controllers/sub/y.json");
    let ignored = found(registry, "IgnoredFile", Severity::Info);
    let mut paths: Vec<_> = ignored
        .iter()
        .map(|f| f.location.path.as_ref().unwrap().as_str().to_owned())
        .collect();
    paths.sort();
    assert_eq!(
        paths,
        ["render_controllers/x.JSON", "render_controllers/x.txt"]
    );
}

/// TOML `vanilla = "forEachIn enumeration order"`: within a pack the files are read in sorted
/// path order, so of two files defining one name the later path wins. README RC.3: every
/// replacement is a `RenderControllerShadowed` info (the game is silent), inside a pack too.
#[test]
fn files_of_one_pack_are_read_in_sorted_path_order_deviation() {
    let a = cow("Geometry.a");
    let b = cow("Geometry.b");
    for files in [
        [
            ("render_controllers/a.json", a.as_str()),
            ("render_controllers/b.json", b.as_str()),
        ],
        [
            ("render_controllers/b.json", b.as_str()),
            ("render_controllers/a.json", a.as_str()),
        ],
    ] {
        let set = stacks(&[&files]);
        let registry = set.resource.render_controllers();
        let cow = registry.get(&id("controller.render.cow")).unwrap();
        assert_eq!(cow.geometry.as_str(), "geometry.b");
        assert_eq!(cow.origin().path.as_str(), "render_controllers/b.json");
        let shadowed = found(registry, "RenderControllerShadowed", Severity::Info);
        assert_eq!(shadowed.len(), 1, "{:?}", all(registry));
        assert_eq!(
            shadowed[0].location.path.as_ref().unwrap().as_str(),
            "render_controllers/a.json"
        );
        // Nothing but that info: the replacement is no warning.
        assert_eq!(all(registry).len(), 1, "{:?}", all(registry));
        assert!(registry.rejected().is_empty());
    }
}

#[test]
fn a_file_is_placed_after_its_first_controller() {
    let mut file = RenderControllerFile::new();
    assert_eq!(
        file.default_path().as_str(),
        "render_controllers/unnamed.json"
    );
    let controller = RenderController::new(
        bedrock_addon::molang::MolangSource::string("geometry.default", 13),
        vec![bedrock_addon::molang::MolangSource::string(
            "texture.default",
            13,
        )],
    );
    file.render_controllers
        .insert("controller.render.cow".to_owned(), controller.clone());
    file.render_controllers
        .insert("controller.render.a".to_owned(), controller);
    assert_eq!(
        file.default_path().as_str(),
        "render_controllers/controller.render.cow.json"
    );
}

/// The game registers `controller.render.Foo` under a name its lower-cased references
/// never reach; our registry folds the key, so it resolves here, with the warning.
#[test]
fn a_name_the_game_cannot_resolve_is_registered_folded_deviation() {
    let text = "{\"format_version\": \"1.8.0\", \"render_controllers\": {\n// it's\n\"controller.render.Foo\": {\"geometry\": \"geometry.default\", \"textures\": [\"texture.default\"]}}}";
    let set = stacks(&[&[("render_controllers/foo.json", text)]]);
    let registry = set.resource.render_controllers();
    let foo = registry.get(&id("controller.render.foo")).unwrap();
    assert_eq!(foo.origin().key.as_deref(), Some("controller.render.Foo"));
    let warnings = found(registry, "CaseFoldingDisabled", Severity::Info);
    assert_eq!(warnings.len(), 1, "{:?}", all(registry));
}

#[test]
fn a_file_whose_controllers_were_all_dropped_is_placed_after_its_first_dropped_one() {
    let text = r#"{"format_version": "1.8.0", "render_controllers": {
        "controller.render.pig": {"geometry": "geometry.pig"},
        "controller.render.cow": {"geometry": "geometry.cow"}}}"#;
    let cx = super::common::rp(1, 21, 100);
    let document = super::common::loaded(super::rows::parse(text, &cx));
    assert!(document.render_controllers.is_empty());
    assert_eq!(document.dropped.len(), 2);
    assert_eq!(
        document.default_path().as_str(),
        "render_controllers/controller.render.cow.json"
    );
    // A loaded controller names the file before any dropped one.
    let mut file = document.value().clone();
    let pig = file.dropped.shift_remove("controller.render.pig").unwrap();
    file.render_controllers
        .insert("controller.render.pig".to_owned(), pig);
    assert_eq!(
        file.default_path().as_str(),
        "render_controllers/controller.render.pig.json"
    );
}

// A file below `render_controllers/` that is no `.json` file is reported from
// the pack's file list; a stack load never asks for its bytes.
#[test]
fn a_stack_load_asks_for_the_json_files_only() {
    use bedrock_addon::load::Collection;

    use crate::responder::Order;

    let ok = file(OK);
    let top: &[(&str, &[u8])] = &[
        ("render_controllers/x.JSON", ok.as_bytes()),
        ("render_controllers/x.txt", b"a note"),
        ("render_controllers/sub/picture.png", b"\x89PNG"),
        ("render_controllers/sub/y.json", ok.as_bytes()),
        ("render_controllers_old/z.txt", b"elsewhere"),
    ];
    let base: &[(&str, &[u8])] = &[("render_controllers/b.bin", b"\0\0")];
    for order in Order::ALL {
        let (set, asked) = crate::requests::stack_load(
            "resources",
            &[top, base],
            &[Collection::RenderControllers],
            order,
        );
        assert_eq!(asked, ["p0/render_controllers/sub/y.json"], "{order:?}");
        let registry = set.resource.render_controllers();
        assert_eq!(registry.len(), 1, "{order:?}");
        // RC-V.1 #1 all the same, lowest pack first and each pack's files in path order.
        let ignored: Vec<(Option<usize>, &str)> = found(registry, "IgnoredFile", Severity::Info)
            .iter()
            .map(|f| {
                let pack = f
                    .location
                    .pack
                    .as_ref()
                    .and_then(bedrock_addon::pack::PackRef::index);
                (pack, f.location.path.as_ref().unwrap().as_str())
            })
            .collect();
        assert_eq!(
            ignored,
            [
                (Some(1), "render_controllers/b.bin"),
                (Some(0), "render_controllers/sub/picture.png"),
                (Some(0), "render_controllers/x.JSON"),
                (Some(0), "render_controllers/x.txt"),
            ],
            "{order:?}"
        );
    }
}

/// Two definitions of one name in the top pack and one below, keeping the shadowed ones:
/// which definitions the registry lists for the name, and its findings in order.
#[test]
fn the_layers_of_a_name_defined_twice_in_a_pack() {
    let a = cow("Geometry.a");
    let b = cow("Geometry.b");
    let c = cow("Geometry.c");
    let set = super::common::stacks_with(
        &[
            &[
                ("render_controllers/a.json", a.as_str()),
                ("render_controllers/b.json", b.as_str()),
            ],
            &[("render_controllers/c.json", c.as_str())],
        ],
        true,
    );
    let registry = set.resource.render_controllers();
    let layers: Vec<(Option<usize>, &str)> = registry
        .layers(&id("controller.render.cow"))
        .map(|d| (d.origin().pack.index(), d.origin().path.as_str()))
        .collect();
    // Today the earlier file of the top pack is left out before the registry sees it.
    assert_eq!(
        layers,
        [
            (Some(0), "render_controllers/b.json"),
            (Some(1), "render_controllers/c.json"),
        ]
    );
    let shadowed: Vec<(Option<usize>, &str)> =
        found(registry, "RenderControllerShadowed", Severity::Info)
            .into_iter()
            .map(|f| {
                (
                    f.location
                        .pack
                        .as_ref()
                        .and_then(bedrock_addon::pack::PackRef::index),
                    f.location.path.as_ref().unwrap().as_str(),
                )
            })
            .collect();
    assert_eq!(
        shadowed,
        [
            (Some(0), "render_controllers/a.json"),
            (Some(1), "render_controllers/c.json"),
        ]
    );
}
