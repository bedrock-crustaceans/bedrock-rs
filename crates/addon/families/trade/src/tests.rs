//! Unit tests of the module's private parts.

#[test]
fn equal_quantities_hash_alike() {
    use std::collections::HashSet;
    let quantities: HashSet<super::Quantity> =
        [super::Quantity::Exact(3), super::Quantity::Exact(3)]
            .into_iter()
            .collect();
    assert_eq!(quantities.len(), 1);
}
