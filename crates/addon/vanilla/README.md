# bedrock_addon_vanilla

The game's own data for the `bedrock_addon` crate family: the identifier lists and the command table generated from the game's script metadata, and the hand-kept name lists the metadata does not carry. Part of [bedrock-rs](https://github.com/bedrock-crustaceans/bedrock-rs).

**This crate is an implementation detail.** Every table is a `&'static` array of plain strings, with no dependencies, so an edit to `bedrock_addon_core` does not recompile the tables. The families ask core's `BuiltIns` provider (`LoadContext::built_ins`), whose default, `bedrock_addon_core::vanilla::GeneratedBuiltIns`, indexes these tables; a host with its own registry passes its own provider. No family depends on this crate; the two readers with no context to ask a provider (`sounds.json`'s event names and entity's goal sounds) read the level sound events through core's re-export, whatever the host. The public path stays `bedrock_addon_core::vanilla` (re-exported as `bedrock_addon::vanilla`).

The files under `src/generated/` are output of `cargo xtask script-meta --samples <dir>`: change `xtask/` and regenerate, never edit by hand.
