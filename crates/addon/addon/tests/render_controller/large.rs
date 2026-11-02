//! One large file costs time linear in its size: the three shapes
//! that must not be quadratic, each loaded at 5 000
//! and at 20 000 entries and held to the crate's one protocol. The texts are built
//! before the clock starts and what was loaded is checked after it stops.

use std::fmt::Write as _;

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::id::RenderControllerId;

use super::common::{count, loaded, stacks, unchecked};
use super::rows::{file, parse};
use crate::cost::{assert_linear_timed_at, timed};

/// The smaller size; the larger is four times it.
const N: usize = 5_000;

#[test]
fn many_controllers_without_textures_are_dropped_in_linear_time() {
    assert_linear_timed_at("controllers without textures", N, |entries| {
        let mut controllers = String::new();
        for n in 0..entries {
            write!(
                controllers,
                r#""controller.render.c{n}": {{"geometry": "geometry.default"}},"#
            )
            .unwrap();
        }
        controllers.push_str(super::rows::OK);
        let text = file(&controllers);
        let cx = unchecked();
        let (document, took) = timed(|| loaded(parse(&text, &cx)));
        assert_eq!(document.render_controllers.len(), 1);
        assert_eq!(
            count(
                document.diagnostics(),
                "MissingRequiredField",
                Severity::Error
            ),
            entries
        );
        // In the file's (sorted) order, as before.
        let first = document
            .diagnostics()
            .iter()
            .find(|f| f.severity == Severity::Error)
            .unwrap();
        assert_eq!(
            first.location.json_path.to_string(),
            "render_controllers/controller.render.c0/textures"
        );
        took
    });
}

#[test]
fn many_array_references_are_checked_in_linear_time() {
    assert_linear_timed_at("references over as many arrays", N, |entries| {
        let mut arrays = String::new();
        let mut textures = String::new();
        for n in 0..entries {
            write!(arrays, r#""Array.a{n}": ["texture.t"],"#).unwrap();
            // Every other reference names an array that is not declared.
            write!(textures, r#""array.a{}[0]","#, n * 2).unwrap();
        }
        arrays.pop();
        textures.pop();
        let text = file(&format!(
            r#""controller.render.big": {{"geometry": "geometry.default", "arrays": {{"textures": {{{arrays}}}}}, "textures": [{textures}]}}"#
        ));
        let cx = unchecked();
        let (document, took) = timed(|| loaded(parse(&text, &cx)));
        assert_eq!(
            count(
                document.diagnostics(),
                "UnresolvedReference",
                Severity::Info
            ),
            entries / 2
        );
        took
    });
}

#[test]
fn a_registry_of_many_controllers_with_findings_loads_in_linear_time() {
    assert_linear_timed_at(
        "a registry of controllers with a warning each",
        N,
        |entries| {
            let mut controllers = String::new();
            for n in 0..entries {
                write!(
                controllers,
                r#""controller.render.c{n}": {{"geometry": "geometry.default", "textures": ["array.missing[0]"]}},"#
            )
            .unwrap();
            }
            controllers.pop();
            let text = file(&controllers);
            // The one file through a stack: the load and the registry of its controllers.
            let (set, took) = timed(|| {
                let set = stacks(&[&[("render_controllers/big.json", text.as_str())]]);
                set.resource.render_controllers();
                set
            });
            let registry = set.resource.render_controllers();
            assert_eq!(registry.len(), entries);
            let last = registry
                .get(&RenderControllerId::parse(format!(
                    "controller.render.c{}",
                    entries - 1
                )))
                .unwrap();
            // Each controller carries its own finding.
            assert_eq!(
                count(last.diagnostics(), "UnresolvedReference", Severity::Info),
                1
            );
            assert_eq!(
                registry
                    .diagnostics()
                    .filter(|f| f.code.name() == "UnresolvedReference")
                    .count(),
                entries
            );
            took
        },
    );
}
