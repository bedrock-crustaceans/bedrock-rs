//! What a link pass is asked to do.

/// Which rules [`StackSet::link_with`](crate::pack::StackSet::link_with) runs.
///
/// Built as `LinkOptions { lints: false, ..Default::default() }`; the default runs every rule.
///
/// ```
/// use bedrock_addon::load::LoadOptions;
/// use bedrock_addon::pack::{LinkOptions, PackRepository, WorldPacks};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let world = WorldPacks {
///     behavior: Vec::new(),
///     resource: Vec::new(),
/// };
/// let stacks = PackRepository::new().stacks(&world, LoadOptions::default());
/// let options = LinkOptions {
///     lints: false,
///     ..Default::default()
/// };
/// let findings = stacks.link_with(&options);
/// assert!(findings.is_empty(), "an empty world links to nothing");
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LinkOptions {
    /// Run the lints: the rules that tell an information, because vanilla keeps the
    /// reference as written and logs nothing. With `false` the pass returns no
    /// information-level finding at all: the lints', and also a miss under a partial view and
    /// the note of a file that is not loaded. A rule that tells only informations may skip
    /// its work; a rule that tells warnings too still runs. A server that links on every
    /// start turns them off.
    pub lints: bool,
    /// The stack is linked as the game's client loads it (`true`, the default), not as a
    /// dedicated server does. A dedicated server reads no client resources (no client entity,
    /// attachable or texture-set loader), so with `false` the rules of what only the client
    /// loads say nothing: R7, the missing-texture check of a client entity, an attachable or
    /// an atomic client entity.
    pub client: bool,
}

/// Every rule, lints included.
impl Default for LinkOptions {
    fn default() -> Self {
        Self {
            lints: true,
            client: true,
        }
    }
}
