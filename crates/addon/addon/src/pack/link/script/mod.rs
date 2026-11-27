//! The link rules of scripting and pack settings.
//!
//! This group adds no finding to `StackSet::link()`, and that is deliberate, not a gap:
//!
//! - The script rules are told by the load. The stack's `scripts()` collection reads the script
//!   module of each pack once, discovers it with the modules of the packs above it and scans it
//!   against the plugin that makes (`script/bundle/load.rs`); a rule the module and the bundle
//!   both state is told by the module, once. None needs both stacks at once: a `script` module
//!   is a behaviour pack's and a `client_script` module a resource pack's, and no import crosses
//!   a pack. A saved value that does not meet its definition is told by `PackSettings::new`, and
//!   a bad Molang expression by core's Molang check.
//! - Three rules are over inputs that are no stack content, so the pass, which sees the two
//!   stacks and nothing else, cannot hold them:
//!   - A config folder that no script module matches is
//!     `ScriptConfig::unmatched_folders_of_modules`, a function of the server's configuration
//!     and the script modules of the stack, started or not
//!     (`stack.scripts().iter().filter_map(|bundle| bundle.module)`); its findings are of the
//!     link phase. (`ScriptConfig::unmatched_folders` takes the plugins, the scripts that
//!     start.)
//!   - A `commands.json` level for a name that is no command is told by `ScriptConfig::parse`:
//!     the command table is the crate's own data and needs no stack.
//!   - A saved setting for a pack the world does not have is `pack::settings::unmatched_entries`,
//!     a function of a saved-settings file and the stacks; its findings are of the link phase.
//!
//! The tests of these rules are `tests/link/script/`.
// No new public link entry and no new field of `LinkOptions`: the three functions above are the
// whole public surface.

pub(super) fn run(_cx: &super::LinkCx<'_>, _out: &mut super::Findings) {}
