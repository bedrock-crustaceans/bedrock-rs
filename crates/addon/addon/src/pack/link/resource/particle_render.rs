//! A particle effect's material is a material of the resource stack.
//!
//! The game looks the material up when the effect is first drawn; a name no material file
//! defines draws nothing and nothing is said at load. Told here as an information at `material`,
//! with the name as written: a material name is compared byte for byte, and a variant is named
//! `<material>.<variant>`.
//!
//! The crate has no list of the game's own materials. The names are judged by the view the
//! pass has of the kind ([`LinkCx::material_view`]): a stack that creates no material and has
//! no host cannot tell a material only the game's resource pack defines (`particles_alpha`)
//! from a missing one, and nothing is said of any material; a stack whose only materials are
//! an add-on's own tells a miss as an information too (the name may be the game's); a partial
//! view only reports when lints are on. A client entity's `materials` is judged the same way.
//! The presence of the member is the load's to tell.

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::id::MaterialId;

use super::super::{At, Findings, LinkCx, kinds, unresolved};
use super::particle::payload;

/// `description.basic_render_parameters.material` of every effect names a material
/// (or a variant of one) that a material file of the stack defines, or one the host says the
/// game has. One that is none is an `UnresolvedReference` information at the member. An empty
/// name is no reference: the load has told the missing material. Nothing is judged in a stack
/// without a view of the materials, and a miss is an information with a partial one.
pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    let view = cx.material_view(out);
    if !view.judges() || (view == super::super::View::Partial && !cx.lints()) {
        return;
    }
    for (_, document) in cx.resource().particles() {
        let name = &document
            .value()
            .description
            .basic_render_parameters
            .material;
        if name.is_empty() {
            continue;
        }
        // Not `LinkCx::material`: on a hit that lookup also finds the place of the material
        // among the keys of its file, which this rule has no use for.
        let known = cx.material_exists(&MaterialId::parse(name.as_str()));
        if !known {
            let mut path = payload();
            path.push("description");
            path.push("basic_render_parameters");
            path.push("material");
            // The material is looked up when the effect is drawn, with no content-log line and
            // no linked-asset reference: the crate reports an information.
            let at = At::new(document.origin(), path, view.severity(Severity::Info));
            out.push(unresolved(at, kinds::MATERIAL, name, None));
        }
    }
}

#[cfg(test)]
mod tests {
    use bedrock_addon_core::diagnostic::Diagnostics;

    use crate::pack::StackSet;
    use crate::pack::link::tests::support::{File, pack, set_of};
    use crate::pack::link::{LinkOptions, link_with_rules};

    /// A world of `n` effects, each drawn with a material of its own: every second one a
    /// material of the one material file's `n` materials, the others nothing's. Everything
    /// the rule reads is loaded.
    fn world(n: usize) -> StackSet {
        use std::fmt::Write as _;
        let mut materials = String::from(r#"{"materials": {"version": "1.0.0""#);
        let mut effects: Vec<(&'static str, String)> = Vec::with_capacity(n);
        for i in 0..n {
            let _ = write!(materials, r#", "m{i}": {{}}"#);
            let material = if i % 2 == 0 { "m" } else { "none" };
            let path: &'static str = Box::leak(format!("particles/e{i}.json").into_boxed_str());
            effects.push((
                path,
                format!(
                    r#"{{"format_version": "1.26.10", "particle_effect": {{"description": {{"identifier": "demo:e{i}", "basic_render_parameters": {{"material": "{material}{i}", "texture": "t"}}}}, "components": {{}}}}}}"#
                ),
            ));
        }
        materials.push_str("}}");
        let mut files: Vec<File<'_>> = vec![
            (
                "materials/common.json",
                Some(r#"[{"path": "materials/my.material"}]"#),
            ),
            ("materials/my.material", Some(materials.as_str())),
        ];
        files.extend(
            effects
                .iter()
                .map(|(path, text)| (*path, Some(text.as_str()))),
        );
        let set = set_of(
            Vec::new(),
            vec![pack(2, "resources", &files)],
            &bedrock_addon_core::load::LoadOptions::default(),
        );
        // The first pass loads the effects and the materials: the timed ones only look up.
        assert_eq!(pass(&set).len(), n / 2);
        set
    }

    fn pass(set: &StackSet) -> Diagnostics {
        link_with_rules(set, LinkOptions::default(), &[super::run])
    }

    /// A hit costs a probe, not the keys of the material's file: `n` effects
    /// that name `n` materials of one file cost `n`.
    #[test]
    fn n_effects_among_n_materials_of_one_file_cost_n() {
        crate::test_cost::assert_linear_at(
            "link: n effects among n materials of one file",
            4_000,
            world,
            |set: &StackSet| {
                let found = pass(set);
                assert!(!found.is_empty());
                found
            },
        );
    }
}
