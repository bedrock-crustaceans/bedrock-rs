//! Vanilla's content-log texts of the spawn-rules loaders, as the loader quotes them. `{}` is
//! where `VanillaText::Formatted` puts each argument (the identifier).

/// The closing line of a spawn-rules file that failed.
pub(crate) const FAILED_RULES: &str = "Failed to load spawn rules";
// The game logs this to its developer log only.
/// An unknown `population_control`.
pub(crate) const NO_POPULATION_CONTROL: &str = "No population control specified for '{}'";

/// `placement.type` not registered.
pub(crate) const PLACEMENT_TYPE: &str =
    "Spawn Rules for 'type' in {} must use a registered placement type predicate";
/// `placement.heightmap` not registered.
pub(crate) const PLACEMENT_HEIGHTMAP: &str =
    "Spawn Rules for 'heightmap' in {} must use a registered heightmap type";
/// `placement.spawn_obstruction_predicate` not registered.
pub(crate) const PLACEMENT_OBSTRUCTION: &str = "Spawn Rules for 'spawn_obstruction_predicate' in {} must use a registered obstruction predicate";
/// `placement.spawn_predicate` not registered.
pub(crate) const PLACEMENT_PREDICATE: &str =
    "Spawn Rules for 'spawn_predicate' in {} must use a registered entity spawn predicate";

/// Every template with arguments, for the arity test.
#[cfg(test)]
pub(crate) const FORMATTED: &[&str] = &[
    NO_POPULATION_CONTROL,
    PLACEMENT_TYPE,
    PLACEMENT_HEIGHTMAP,
    PLACEMENT_OBSTRUCTION,
    PLACEMENT_PREDICATE,
];

#[cfg(test)]
mod tests {
    use super::FORMATTED;

    #[test]
    fn every_formatted_text_has_one_placeholder() {
        for template in FORMATTED {
            assert_eq!(template.matches("{}").count(), 1, "{template}");
        }
    }
}
