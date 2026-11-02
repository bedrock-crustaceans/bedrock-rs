//! The whole of a stack load at a size a failure can be read at: what three packs of particle
//! files combine to. That each part of a load is linear in its entries is measured where the
//! part is, on inputs built beforehand (`named_map`, `read_components`, the map, list and
//! gradient readers and `combine` in `crates/addon/families/particle/src/`); a stack load is mostly
//! its walk and its JSON parse, and a time bound on it sees neither of them go quadratic.

use bedrock_addon::id::ParticleId;
use bedrock_addon::pack::{PackStack, Resource};

use super::common::{CEREAL, document, options, pack};

/// A stack: `COUNT` files in one pack, each its own effect, under a pack that replaces half
/// of them and one whose files all fail.
#[test]
fn a_stack_of_three_packs_keeps_each_effect_of_the_highest_pack_that_loads_it() {
    const COUNT: usize = 200;
    let spark = |identifier: usize, particles: u32| {
        document(
            CEREAL,
            &format!(
                r#""components": {{"minecraft:emitter_rate_instant": {{"num_particles": {particles}}}}}"#
            ),
        )
        .replace("demo:spark", &format!("demo:p{identifier}"))
    };
    let bottom = (0..COUNT)
        .map(|i| (format!("particles/p{i}.json"), spark(i, 1)))
        .collect();
    let middle = (0..COUNT / 2)
        .map(|i| (format!("particles/q{i}.json"), spark(i * 2, 2)))
        .collect();
    let top = (0..COUNT / 2)
        .map(|i| {
            (
                format!("particles/r{i}.json"),
                spark(i, 3).replace("1.26.10", "1.0.0"),
            )
        })
        .collect();
    let packs = vec![
        pack(1, "top", top),
        pack(2, "middle", middle),
        pack(3, "bottom", bottom),
    ];
    let stack = PackStack::<Resource>::of(packs, options()).expect("a stack");
    let particles = stack.particles();
    assert_eq!(particles.len(), COUNT);
    assert_eq!(particles.rejected().len(), COUNT / 2);
    for i in 0..COUNT {
        let effect = particles
            .get(&ParticleId::parse(format!("demo:p{i}")))
            .unwrap();
        let from = if i % 2 == 0 { "middle" } else { "bottom" };
        assert_eq!(effect.origin().pack.name(), from, "demo:p{i}");
    }
}
