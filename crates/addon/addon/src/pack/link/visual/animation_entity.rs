//! A particle or sound `effect` of a resource-pack animation, or of a state
//! of a resource-pack animation controller, is a key of `particle_effects` / `sound_effects`
//! of the client entities that play it. The game looks the key up when the effect fires and
//! logs nothing when the file loads: a note.
//!
//! Both sides are read in lower case. An animation no client entity plays is not checked; an
//! effect is told once, at the animation, however many of the entities that play it lack the
//! key. The cost is, per animation, the effect maps of its players and its own effects.

use std::cell::OnceCell;
use std::collections::{HashMap, HashSet};

use bedrock_addon_core::diagnostic::{JsonPath, Severity};
use bedrock_addon_core::document::Origin;

use super::super::{At, Findings, LinkCx, unresolved};
use super::user::{Family, User, lowered_keys};
use crate::families::animation::{OneOrMany, ParticleEffect, SoundEffect};

/// The kind of a finding about `particle_effects` of a client entity.
const PARTICLE_EFFECT: &str = "particle effect";
/// The kind of a finding about `sound_effects` of a client entity.
const SOUND_EFFECT: &str = "sound effect";

/// The effect keys of one client entity, in lower case.
#[derive(Debug)]
struct Keys {
    particles: HashSet<String>,
    sounds: HashSet<String>,
}

/// The client entities that play each animation and animation controller, by the name they
/// give it (in lower case, as the game reads `animations` values).
#[derive(Debug)]
struct Players {
    keys: Vec<Keys>,
    of: HashMap<String, Vec<usize>>,
}

impl Players {
    fn new(entities: &[User<'_>]) -> Self {
        let mut players = Self {
            keys: Vec::new(),
            of: HashMap::new(),
        };
        for user in entities {
            if user.family != Family::ClientEntity {
                continue;
            }
            let Some(animations) = user.animations else {
                continue;
            };
            let index = players.keys.len();
            players.keys.push(Keys {
                particles: lowered_keys(user.particle_effects),
                sounds: lowered_keys(user.sound_effects),
            });
            for value in animations.values() {
                let played = players.of.entry(user.folded(value.as_str()).into_owned());
                let played = played.or_default();
                if played.last() != Some(&index) {
                    played.push(index);
                }
            }
        }
        players
    }

    /// How many of `players` have each effect key: what [`Players::check`] asks, built once
    /// per animation at the cost of its players' maps.
    fn counts(&self, players: &[usize]) -> Counts<'_> {
        let mut counts = Counts {
            players: players.len(),
            particles: HashMap::new(),
            sounds: HashMap::new(),
        };
        for keys in players.iter().filter_map(|player| self.keys.get(*player)) {
            for key in &keys.particles {
                *counts.particles.entry(key).or_default() += 1;
            }
            for key in &keys.sounds {
                *counts.sounds.entry(key).or_default() += 1;
            }
        }
        counts
    }
}

/// How many of the players of one animation have each effect key.
#[derive(Debug)]
struct Counts<'p> {
    players: usize,
    particles: HashMap<&'p str, usize>,
    sounds: HashMap<&'p str, usize>,
}

impl Counts<'_> {
    /// Tells the effect `effect` at `path` of the document of `origin` when one of the
    /// players lacks it.
    fn check(
        &self,
        sound: bool,
        effect: &str,
        origin: &Origin,
        path: JsonPath,
        out: &mut Findings,
    ) {
        let key = effect.to_ascii_lowercase();
        let have = if sound { &self.sounds } else { &self.particles };
        if have.get(key.as_str()).copied().unwrap_or(0) < self.players {
            let kind = if sound { SOUND_EFFECT } else { PARTICLE_EFFECT };
            let at = At::new(origin, path, Severity::Info);
            out.push(unresolved(at, kind, &key, None));
        }
    }
}

/// The path of the member `member` of the definition `key` in a file whose definitions are
/// under `root`.
fn below(root: &str, key: &str, member: &[&str]) -> JsonPath {
    let mut path = JsonPath::new();
    path.push(root);
    path.push(key);
    for step in member {
        path.push(step);
    }
    path
}

pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings, entities: &[User<'_>]) {
    if !cx.lints() {
        return;
    }
    let players = Players::new(entities);
    if players.of.is_empty() {
        return;
    }
    // The timelines of an animation.
    for (id, document) in cx.resource().animations() {
        let Some(played_by) = players.of.get(id.as_str()) else {
            continue;
        };
        // Counted on the first effect: most animations have none.
        let counted = OnceCell::new();
        let counts = || counted.get_or_init(|| players.counts(played_by));
        let origin = document.origin();
        let key = origin.key.as_deref().unwrap_or(id.as_str());
        let animation = document.value();
        for (time, effects) in animation.particle_effects.iter().flatten() {
            let at = below("animations", key, &["particle_effects", time]);
            each(effects, &at, |effect: &ParticleEffect, path| {
                counts().check(false, &effect.effect, origin, path, out);
            });
        }
        for (time, effects) in animation.sound_effects.iter().flatten() {
            let at = below("animations", key, &["sound_effects", time]);
            each(effects, &at, |effect: &SoundEffect, path| {
                counts().check(true, &effect.effect, origin, path, out);
            });
        }
    }
    // The states of an animation controller.
    for (id, document) in cx.resource().animation_controllers() {
        let Some(played_by) = players.of.get(id.as_str()) else {
            continue;
        };
        // Counted on the first effect: most animations have none.
        let counted = OnceCell::new();
        let counts = || counted.get_or_init(|| players.counts(played_by));
        let origin = document.origin();
        let key = origin.key.as_deref().unwrap_or(id.as_str());
        for (name, state) in &document.value().states {
            let at = below("animation_controllers", key, &["states", name]);
            for (index, effect) in state.particle_effects.iter().flatten().enumerate() {
                let path = element(&at, "particle_effects", index);
                counts().check(false, &effect.effect, origin, path, out);
            }
            for (index, effect) in state.sound_effects.iter().flatten().enumerate() {
                let path = element(&at, "sound_effects", index);
                counts().check(true, &effect.effect, origin, path, out);
            }
        }
    }
}

/// Calls `tell` for each effect under one time key with the path of its `effect` member:
/// the one value, or each element of the array.
fn each<T>(effects: &OneOrMany<T>, at: &JsonPath, mut tell: impl FnMut(&T, JsonPath)) {
    match effects {
        OneOrMany::One(effect) => {
            let mut path = at.clone();
            path.push("effect");
            tell(effect, path);
        }
        OneOrMany::Many(list) => {
            for (index, effect) in list.iter().enumerate() {
                let mut path = at.clone();
                path.push_index(index);
                path.push("effect");
                tell(effect, path);
            }
        }
    }
}

/// The path of `effect` of the element `index` of the list `member` below `at`.
fn element(at: &JsonPath, member: &str, index: usize) -> JsonPath {
    let mut path = at.clone();
    path.push(member);
    path.push_index(index);
    path.push("effect");
    path
}
