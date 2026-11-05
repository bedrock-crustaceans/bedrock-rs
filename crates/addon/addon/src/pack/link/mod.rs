//! The link pass: the references between the definitions of a world's two pack stacks, of two
//! families or of one, resolved after the load. `StackSet::link` is the entry; it reads the
//! collections through the stacks' own accessors (which load what is not loaded yet), changes
//! none of them and returns findings.
//!
//! The frame is here: the context of one pass ([`LinkCx`]) with one lookup per kind of target,
//! the resolvers of the shared descriptor types, the constructors of the findings and the sink
//! they go into ([`Findings`]). The rules are in six group modules, one per rule package, each
//! with one `run(&LinkCx, &mut Findings)`; inside a group a rule relating the modules `a`
//! and `b` is the file `<a>_<b>.rs` under `#[cfg(all(feature = "a", feature = "b"))]`, `a`
//! being the module that holds the reference (a pair inside one module is `<a>.rs` under
//! `#[cfg(feature = "a")]`), so that with either feature off the rule does not run and
//! nothing is reported.
//!
//! No rule reads the arguments of a Molang expression: a reference carried as the argument
//! of a query (`q.block_state('s')`, `q.property('p')`, `q.had_component_group('g')`, the
//! tags of `q.any_tag`) is resolved by the game when the expression runs, and by nothing
//! here. Rules of the `visual` group do read Molang text, in two ways: a render controller's
//! expression that is exactly `geometry.<name>`, `texture.<name>` or `material.<name>` is
//! taken as the slot it names, and a controller's `light_color_multiplier` is asked whether
//! it is a constant (by its text and, with the `molang` feature, by compiling it).
//!
//! What a caller sees of the pass:
//!
//! - **Findings only.** Every finding is of `Phase::Link`; the groups run in a fixed order
//!   (entity, content, world, visual, resource, script), so two passes over the same stacks
//!   return the same findings in the same order. No registry is changed.
//! - **Severity** is graded by what the rule records of the game's handling of the document
//!   holding the reference (nothing here runs the game): an error
//!   where it drops the document or definition, a warning where it drops the entry or member,
//!   puts a default in its place or logs the miss and keeps the value, an information where it
//!   keeps the reference as written and logs nothing. With `LinkOptions { lints: false, ..Default::default() }`
//!   the sink drops every information: the lints, and also a miss under a partial view and
//!   `FileNotLoaded`, which are informations without being lints. A rule that tells only
//!   informations may ask `lints()` first and skip its work; one that tells warnings too
//!   runs and leaves the dropping to the sink.
//! - **The game's linked-asset check** logs every reference it holds and cannot resolve as
//!   `<breadcrumb>:  Missing referenced asset <name>` or, for a file,
//!   `<breadcrumb>:  Invalid asset path <name>`, always at the content log's error level, and
//!   drops nothing. A rule that mirrors such a reference carries that line as its text
//!   ([`Validator`]) and its finding is an error (an information where the view of the game's
//!   content is partial). Where the family logs a line of its own for the same miss, the
//!   validator's line is a second finding, also an error, at the member the validator names.
//! - **The codes of a miss**, for most rules: `UnresolvedReference` (the name resolves to nothing),
//!   `InvalidReference` (it resolves to something the use cannot accept), `ReferenceCycle`,
//!   `TextureNotFound`, `UnknownSound`, `UnresolvedLangKey`; the kind and the name are the
//!   finding's `DiagnosticDetail::Reference`.
//! - **The game's own content** is answered by the host (`LoadOptions::built_ins`), which is
//!   believed, yes or no, and without a host by the crate's lists for the kinds that have
//!   one; some names stand whatever a host says. The `lookup` module's documentation has
//!   the lists. A reference to a `material` or an `atlas key` is judged by the view the pass
//!   has of the kind (`lookup::View`).
//! - **Bytes** a rule asks the pass for (`LinkCx::bytes`) are given only where they are
//!   resident: for a file that is indexed and not loaded the rule gets none, and one
//!   `FileNotLoaded` information says so (with the lints off, nothing does). The lookup of a
//!   localisation key reads `.lang` files itself and tells nothing of one that is not
//!   loaded.
//! - The sixth group, `script`, runs no rule: the script rules are told by the load, and
//!   the three over inputs that are no stack content are functions of their own
//!   (`ScriptConfig::unmatched_folders` with `unmatched_folders_of_modules`,
//!   `ScriptConfig::parse`, `pack::settings::unmatched_entries`).
// The six groups run in a fixed order, so two passes over the same stacks give the same
// findings in the same order.
mod content;
mod entity;
mod resource;
mod script;
mod visual;
mod world;

mod cx;
mod finding;
/// A unit test of the frame written where a rule group's own unit tests are: one level
/// below `link`, beside the six groups.
#[cfg(test)]
#[path = "tests_group_shape.rs"]
mod group_shape;
mod lookup;
mod options;
mod shared;
#[cfg(test)]
mod tests;

pub use options::LinkOptions;

#[allow(
    unused_imports,
    reason = "the vocabulary of the rule packages: each uses the part its rows need"
)]
pub(crate) use cx::LinkCx;
#[allow(
    unused_imports,
    reason = "the vocabulary of the rule packages: each uses the part its rows need"
)]
pub(crate) use finding::{
    At, Validator, cycle, invalid, texture_not_found, unknown_sound, unresolved,
    unresolved_lang_key, validator_miss, validator_text,
};
#[allow(
    unused_imports,
    reason = "the vocabulary of the rule packages: each uses the part its rows need"
)]
pub(crate) use lookup::{Names, Target, View, canonical, kinds};
#[allow(
    unused_imports,
    reason = "the vocabulary of the rule packages: each uses the part its rows need"
)]
#[cfg(feature = "item")]
pub(crate) use shared::item_descriptor;
#[allow(
    unused_imports,
    reason = "the vocabulary of the rule packages: each uses the part its rows need"
)]
#[cfg(any(feature = "block", feature = "item"))]
pub(crate) use shared::{Member, States};
#[allow(
    unused_imports,
    reason = "the vocabulary of the rule packages: each uses the part its rows need"
)]
#[cfg(feature = "block")]
pub(crate) use shared::{block_descriptor, block_specifier};

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, Phase, Severity};

use super::StackSet;

/// What a pass has found so far, in the order the rules told it.
#[derive(Debug)]
pub(crate) struct Findings {
    /// Whether the lints run.
    lints: bool,
    list: Diagnostics,
}

impl Findings {
    fn new(options: LinkOptions) -> Self {
        Self {
            lints: options.lints,
            list: Diagnostics::default(),
        }
    }

    /// Whether the lints run. A rule that tells only informations may ask before it does any
    /// work; one that tells warnings too runs, and the sink drops its informations.
    #[allow(dead_code, reason = "for the rule packages")]
    pub(crate) fn lints(&self) -> bool {
        self.lints
    }

    /// Tells `finding`. It is a finding of the link phase whatever phase it was built with; an
    /// information is dropped when the lints are off.
    pub(crate) fn push(&mut self, mut finding: Diagnostic) {
        if finding.severity <= Severity::Info && !self.lints {
            return;
        }
        finding.phase = Phase::Link;
        self.list.push(finding);
    }

    fn finish(self) -> Diagnostics {
        self.list
    }
}

impl Extend<Diagnostic> for Findings {
    /// Tells every finding of `findings`, in order ([`push`](Findings::push)).
    fn extend<I: IntoIterator<Item = Diagnostic>>(&mut self, findings: I) {
        for finding in findings {
            self.push(finding);
        }
    }
}

/// The link pass over `set`: the six groups in their fixed order.
pub(crate) fn link(set: &StackSet, options: LinkOptions) -> Diagnostics {
    let cx = LinkCx::new(set, options);
    let mut out = Findings::new(options);
    entity::run(&cx, &mut out);
    content::run(&cx, &mut out);
    world::run(&cx, &mut out);
    visual::run(&cx, &mut out);
    resource::run(&cx, &mut out);
    script::run(&cx, &mut out);
    out.finish()
}

/// One rule, or one group of rules, as the pass calls it.
#[cfg(test)]
pub(crate) type Rule = for<'a, 'b, 'c> fn(&'a LinkCx<'b>, &'c mut Findings);

/// A pass over `set` that runs `rules` in place of the six groups: how the frame is tested
/// with rules made up for the test.
#[cfg(test)]
pub(crate) fn link_with_rules(set: &StackSet, options: LinkOptions, rules: &[Rule]) -> Diagnostics {
    let cx = LinkCx::new(set, options);
    let mut out = Findings::new(options);
    for rule in rules {
        rule(&cx, &mut out);
    }
    out.finish()
}
