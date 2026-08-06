use facet::Facet;


#[derive(Debug, Copy, Clone, PartialEq, Eq, Facet)]
#[facet(rename_all = "PascalCase")]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct Beehive {
    /// Whether the beehive should spawn bees.
    pub should_spawn_bees: bool,
}
