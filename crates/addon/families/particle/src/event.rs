//! Events (`particle_effect/events`): what an effect does when one of its components fires an
//! event by name.

#![allow(
    clippy::single_match_else,
    reason = "the two arms are the two outcomes of a read: the value, or the line about it"
)]

use super::codes;
use super::derived::{
    EVENT, KEPT, LEGACY, STRICT, elements, legacy, line, note, object, refuse, wrong,
};
use super::read::defaults;
use bedrock_addon_core::diagnostic::Code;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::molang::Molang;
use bedrock_addon_core::schema::{self, Schema};

/// What kind of effect an event starts (`particle_effect.type`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, facet::Facet)]
#[non_exhaustive]
#[repr(u8)]
pub enum ParticleEffectType {
    /// A single particle at the event's position (JSON `"particle"`).
    #[facet(rename = "particle")]
    Particle,
    /// A single particle that takes the velocity of what fired the event (JSON
    /// `"particle_with_velocity"`).
    #[facet(rename = "particle_with_velocity")]
    ParticleWithVelocity,
    /// A new emitter at the event's position (JSON `"emitter"`).
    #[facet(rename = "emitter")]
    Emitter,
    /// A new emitter bound to the same entity (JSON `"emitter_bound"`).
    #[facet(rename = "emitter_bound")]
    EmitterBound,
}

bedrock_addon_core::impl_no_molang!(ParticleEffectType);
bedrock_addon_core::impl_schema_leaf!(ParticleEffectType);

impl ParticleEffectType {
    /// The type a JSON spelling names, compared byte for byte; the reverse of
    /// [`ParticleEffectType::as_str`].
    #[must_use]
    pub fn from_name(text: &str) -> Option<Self> {
        [
            Self::Particle,
            Self::ParticleWithVelocity,
            Self::Emitter,
            Self::EmitterBound,
        ]
        .into_iter()
        .find(|kind| kind.as_str() == text)
    }

    /// The JSON spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Particle => "particle",
            Self::ParticleWithVelocity => "particle_with_velocity",
            Self::Emitter => "emitter",
            Self::EmitterBound => "emitter_bound",
        }
    }
}

/// `particle_effect.type`, with the legacy parser's line for a value that is no type.
fn effect_type(node: &Node<'_>, read: &mut Read<'_>) -> Option<ParticleEffectType> {
    let text = node.value().as_string().map(facet_value::VString::as_str);
    let kind = text.and_then(ParticleEffectType::from_name);
    if kind.is_none() {
        if text.is_some() && !legacy(read) {
            refuse(read, node.path(), Code::from(&core_codes::INVALID_VALUE));
        } else {
            wrong(read, node, Some(&codes::TYPE_MISSING));
        }
    }
    kind
}

/// `particle_effect` of an event: another effect the event starts.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
#[facet(schema::unknown = "report", schema::molang = KEPT)]
pub struct VisualEffectEvent {
    /// The identifier of the effect (JSON `effect`, required).
    #[facet(
        default,
        schema::era(
            LEGACY,
            missing = codes::EFFECT_MISSING,
            wrong_type = codes::EFFECT_MISSING
        ),
        schema::era(STRICT, missing = core_codes::MISSING_REQUIRED_FIELD)
    )]
    pub effect: String,
    /// How it is started (JSON `type`, required).
    #[facet(
        rename = "type",
        default = ParticleEffectType::Particle,
        schema::read_with = effect_type,
        schema::era(LEGACY, missing = codes::TYPE_MISSING),
        schema::era(STRICT, missing = core_codes::MISSING_REQUIRED_FIELD)
    )]
    pub kind: ParticleEffectType,
    /// Molang run on the new emitter before it starts (JSON `pre_effect_expression`;
    /// default 0).
    #[facet(schema::era(LEGACY, molang = EVENT))]
    pub pre_effect_expression: Option<Molang>,
    /// Unknown members.
    #[facet(skip_deserializing)]
    pub extra: UnknownMembers,
}

defaults!(VisualEffectEvent {
    pre_effect_expression: Molang = "pre_effect_expression", Molang::Const(0.0);
});

/// `sound_effect` of an event: a sound the event plays.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
#[facet(schema::unknown = "report")]
pub struct SoundEffectEvent {
    /// The level sound event (JSON `event_name`, required).
    #[facet(
        default,
        schema::era(LEGACY, missing = codes::SOUND_NAME, wrong_type = codes::SOUND_NAME),
        schema::era(STRICT, missing = core_codes::MISSING_REQUIRED_FIELD)
    )]
    pub event_name: String,
    /// Unknown members.
    #[facet(skip_deserializing)]
    pub extra: UnknownMembers,
}

/// One node of an event: it may run Molang, start an effect, play a sound, log a line, run
/// further nodes in order, and pick one of several nodes at random.
///
/// JSON: an object with any of `sequence`, `randomize`, `expression`, `particle_effect`,
/// `sound_effect`, `log`.
///
/// Build one from [`EventNode::default`] and assign the members it has: the type takes its
/// nested nodes apart itself when it is dropped, so that a long chain does not use the
/// stack, and a type with a `Drop` impl has no `..` update syntax.
///
/// Nodes nest through `sequence` and `randomize`. A tree read from a file is at most as deep
/// as the loader's nesting limit lets a file be (128 levels of JSON, which is 62 nodes below
/// an event), and the writer refuses a deeper one with its path. A tree built in memory may
/// be deeper: it is dropped without a stack frame per level, but `Clone`, `PartialEq` and
/// `Debug` recurse once per level, as derived impls do, so a chain of many thousands of
/// nodes is one to drop or to hand to the writer, not to clone, compare or print.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
#[facet(
    schema::unknown = "report",
    schema::molang = KEPT,
    schema::after_read = sections
)]
pub struct EventNode {
    /// Nodes run one after the other (JSON `sequence`).
    #[facet(schema::read_with = sequence)]
    pub sequence: Option<Vec<EventNode>>,
    /// Nodes of which one is picked by weight (JSON `randomize`).
    #[facet(schema::read_with = randomize)]
    pub randomize: Option<Vec<RandomEventNode>>,
    /// Molang run on the emitter that fired the event (JSON `expression`).
    #[facet(schema::era(LEGACY, molang = EVENT))]
    pub expression: Option<Molang>,
    /// An effect to start (JSON `particle_effect`).
    #[facet(schema::era(LEGACY, wrong_type = codes::EFFECT_SECTION))]
    pub particle_effect: Option<VisualEffectEvent>,
    /// A sound to play (JSON `sound_effect`).
    #[facet(schema::era(LEGACY, wrong_type = codes::SOUND_SECTION))]
    pub sound_effect: Option<SoundEffectEvent>,
    /// A line for the content log (JSON `log`).
    pub log: Option<String>,
    /// Unknown members.
    #[facet(skip_deserializing)]
    pub extra: UnknownMembers,
}

/// One option of `randomize`: a node and its weight.
///
/// JSON: the node's members and `weight` in one object.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
#[facet(schema::unknown = "report", schema::after_read = option_sections)]
pub struct RandomEventNode {
    /// The option's share of the picks (JSON `weight`, required from format 1.20.80).
    #[facet(
        default = 0.0,
        schema::guard = weighted,
        schema::era(STRICT, missing = core_codes::MISSING_REQUIRED_FIELD)
    )]
    pub weight: f32,
    /// What the option does.
    #[facet(flatten)]
    pub node: EventNode,
}

impl EventNode {
    /// `expression`, or its default 0.
    pub fn expression(&self) -> Molang {
        self.expression.clone().unwrap_or(Molang::Const(0.0))
    }

    /// `log`, or its default `""`.
    pub fn log(&self) -> &str {
        self.log.as_deref().unwrap_or_default()
    }

    /// Takes the nodes directly below this one out of it.
    fn take_nodes(&mut self) -> Vec<EventNode> {
        let mut nodes = self.sequence.take().unwrap_or_default();
        nodes.extend(
            self.randomize
                .take()
                .unwrap_or_default()
                .into_iter()
                .map(|mut option| std::mem::take(&mut option.node)),
        );
        nodes
    }
}

/// `sequence`: an element that is no object is the legacy parser's line of an event node.
fn sequence(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<EventNode>> {
    elements(node, read, Some(&codes::SEQUENCE), |element, read| {
        object(element, read, Some(&codes::EVENT_NO_DATA))
    })
}

/// `randomize`, as [`sequence`].
fn randomize(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<RandomEventNode>> {
    elements(node, read, Some(&codes::RANDOM), |element, read| {
        object(element, read, Some(&codes::EVENT_NO_DATA))
    })
}

/// The legacy parser makes no effect without both of its required members, and no sound
/// without a name. A section it makes nothing of goes whole; where the file has it as an
/// object, the crate notes that at the section, at the information level: the game's line
/// at its member does not say that the other members go with it.
///
/// An option's node is flattened into it, and its `after_read` would run before the option
/// tells its unknown members; the parser tells them first, so the option drops the sections
/// itself ([`option_sections`]).
fn sections(node: &mut EventNode, at: &Node<'_>, read: &mut Read<'_>) {
    let path = at.path();
    let option = path.last_key().is_none()
        && path.prefix(path.len().saturating_sub(1)).last_key() == Some("randomize");
    if !option {
        drop_sections(node, at, read);
    }
}

/// [`sections`] of an option's node, after the option told its unknown members.
fn option_sections(option: &mut RandomEventNode, at: &Node<'_>, read: &mut Read<'_>) {
    drop_sections(&mut option.node, at, read);
}

/// What [`sections`] drops.
fn drop_sections(node: &mut EventNode, at: &Node<'_>, read: &mut Read<'_>) {
    let Some(m) = at.value().as_object().filter(|_| legacy(read)) else {
        return;
    };
    let section = |key: &str, members: &[&str]| {
        m.get(key)
            .and_then(Value::as_object)
            .is_some_and(|section| {
                members
                    .iter()
                    .all(|member| section.get(member).is_some_and(Value::is_string))
            })
    };
    let left_out = |key: &str, read: &mut Read<'_>| {
        if m.get(key).is_some_and(Value::is_object) {
            note(read, &at.path().child(key), &core_codes::DROPPED_FIELD);
        }
    };
    // A legacy type the reader had to replace is no effect.
    let written_kind = |kind: ParticleEffectType| {
        m.get("particle_effect")
            .and_then(Value::as_object)
            .and_then(|section| section.get("type"))
            .and_then(Value::as_string)
            .is_some_and(|text| text.as_str() == kind.as_str())
    };
    if !section("particle_effect", &["effect", "type"])
        || node
            .particle_effect
            .as_ref()
            .is_some_and(|effect| !written_kind(effect.kind))
    {
        node.particle_effect = None;
        left_out("particle_effect", read);
    }
    if !section("sound_effect", &["event_name"]) {
        node.sound_effect = None;
        left_out("sound_effect", read);
    }
}

/// The legacy parser skips an option without a number for a weight, or with a negative one,
/// with every member it has: its line is at the weight alone, and the crate notes the skip at
/// the option, at the information level. A 1.20.80 document requires the member and has no
/// bound.
fn weighted(option: &Node<'_>, read: &mut Read<'_>) -> bool {
    if !legacy(read) {
        return true;
    }
    let at = option.path().child("weight");
    let written = option.value().as_object().and_then(|m| m.get("weight"));
    match written
        .and_then(Value::as_number)
        .map(facet_value::VNumber::to_f64_lossy)
    {
        Some(weight) if weight >= 0.0 => return true,
        Some(weight) => {
            #[allow(
                clippy::cast_possible_truncation,
                reason = "the nearest f32 is the reading"
            )]
            if !(weight as f32).is_finite() {
                note(read, &at, &core_codes::INVALID_VALUE);
            }
            line(read, &at, &codes::WEIGHT_NEGATIVE);
        }
        None => line(read, &at, &codes::WEIGHT),
    }
    note(read, option.path(), &core_codes::DROPPED_FIELD);
    false
}

impl Drop for EventNode {
    fn drop(&mut self) {
        // Every node is taken apart on this one list; a node dropped at the end of a turn of
        // the loop has no node below it left, so its own drop goes no deeper.
        let mut pending = self.take_nodes();
        while let Some(mut node) = pending.pop() {
            pending.append(&mut node.take_nodes());
        }
    }
}

#[cfg(test)]
mod from_name_tests {
    use super::*;

    /// Every type reads back from its own JSON spelling, and nothing else names one.
    #[test]
    fn an_effect_type_reads_back_from_its_spelling() {
        for kind in [
            ParticleEffectType::Particle,
            ParticleEffectType::ParticleWithVelocity,
            ParticleEffectType::Emitter,
            ParticleEffectType::EmitterBound,
        ] {
            assert_eq!(ParticleEffectType::from_name(kind.as_str()), Some(kind));
        }
        assert_eq!(ParticleEffectType::from_name("Emitter"), None);
    }
}
