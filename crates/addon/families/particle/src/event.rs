//! Events (`particle_effect/events`): what an effect does when one of its components fires an
//! event by name.

#![allow(
    clippy::single_match_else,
    reason = "the two arms are the two outcomes of a read: the value, or the line about it"
)]

use facet_value::{VArray, VObject};

use super::codes;
use super::read::{Era, Members, Node, Opt, Reader, Writer, float, object, opt, put_opt, walk};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::json::{EncodeError, MAX_JSON_DEPTH, Unrepresentable, Value};
use bedrock_addon_core::molang::{Molang, MolangVisitor, Step};

/// What kind of effect an event starts (`particle_effect.type`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ParticleEffectType {
    /// A single particle at the event's position (JSON `"particle"`).
    Particle,
    /// A single particle that takes the velocity of what fired the event (JSON
    /// `"particle_with_velocity"`).
    ParticleWithVelocity,
    /// A new emitter at the event's position (JSON `"emitter"`).
    Emitter,
    /// A new emitter bound to the same entity (JSON `"emitter_bound"`).
    EmitterBound,
}

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

impl Node for ParticleEffectType {
    fn read(value: &Value, r: &mut Reader<'_>, o: Opt) -> Option<Self> {
        let text = value.as_string().map(facet_value::VString::as_str);
        let read = text.and_then(Self::from_name);
        if read.is_none() {
            if text.is_some() && !r.legacy() {
                r.error(DiagnosticCode::from(&core_codes::INVALID_VALUE));
            } else {
                r.wrong_type(value, o.wrong);
            }
        }
        read
    }

    fn fallback(_: Opt) -> Self {
        Self::Particle
    }

    fn write(&self, _: &mut Writer) -> Result<Value, EncodeError> {
        Ok(Value::from(self.as_str()))
    }
}

/// A Molang member of an event: 0 when absent. The legacy parser reads it only from a string
/// and keeps an expression that does not compile.
const EXPRESSION: Opt = Opt::NONE.event();

object! {
    /// `particle_effect` of an event: another effect the event starts.
    pub struct VisualEffectEvent {
        /// The identifier of the effect (JSON `effect`, required).
        req effect: String = "effect",
            Opt::NONE.missing(&codes::EFFECT_MISSING).wrong(&codes::EFFECT_MISSING);
        /// How it is started (JSON `type`, required).
        req kind: ParticleEffectType = "type",
            Opt::NONE.missing(&codes::TYPE_MISSING).wrong(&codes::TYPE_MISSING);
        /// Molang run on the new emitter before it starts (JSON `pre_effect_expression`;
        /// default 0).
        opt pre_effect_expression: Molang = "pre_effect_expression", EXPRESSION,
            default Molang::Const(0.0);
    }
}

object! {
    /// `sound_effect` of an event: a sound the event plays.
    pub struct SoundEffectEvent {
        /// The level sound event (JSON `event_name`, required).
        req event_name: String = "event_name",
            Opt::NONE.missing(&codes::SOUND_NAME).wrong(&codes::SOUND_NAME);
    }
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
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EventNode {
    /// Nodes run one after the other (JSON `sequence`).
    pub sequence: Option<Vec<EventNode>>,
    /// Nodes of which one is picked by weight (JSON `randomize`).
    pub randomize: Option<Vec<RandomEventNode>>,
    /// Molang run on the emitter that fired the event (JSON `expression`).
    pub expression: Option<Molang>,
    /// An effect to start (JSON `particle_effect`).
    pub particle_effect: Option<VisualEffectEvent>,
    /// A sound to play (JSON `sound_effect`).
    pub sound_effect: Option<SoundEffectEvent>,
    /// A line for the content log (JSON `log`).
    pub log: Option<String>,
    /// Unknown members.
    pub extra: UnknownMembers,
}

/// One option of `randomize`: a node and its weight.
///
/// JSON: the node's members and `weight` in one object.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RandomEventNode {
    /// The option's share of the picks (JSON `weight`, required from format 1.20.80).
    pub weight: f32,
    /// What the option does.
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

    fn known(key: &str) -> bool {
        matches!(
            key,
            "sequence" | "randomize" | "expression" | "particle_effect" | "sound_effect" | "log"
        )
    }

    fn known_in_randomize(key: &str) -> bool {
        key == "weight" || Self::known(key)
    }

    fn read_node(m: Members<'_>, random: bool, r: &mut Reader<'_>) -> Self {
        let known = if random {
            Self::known_in_randomize
        } else {
            Self::known
        };
        let legacy = r.legacy();
        let mut node = Self {
            sequence: opt(m, "sequence", Opt::NONE.wrong(&codes::SEQUENCE), r),
            randomize: opt(m, "randomize", Opt::NONE.wrong(&codes::RANDOM), r),
            expression: opt(m, "expression", EXPRESSION, r),
            particle_effect: opt(
                m,
                "particle_effect",
                Opt::NONE.wrong(&codes::EFFECT_SECTION),
                r,
            ),
            sound_effect: opt(m, "sound_effect", Opt::NONE.wrong(&codes::SOUND_SECTION), r),
            log: opt(m, "log", Opt::NONE, r),
            extra: m.extra(known, r),
        };
        if legacy {
            // The legacy parser makes no effect without both of its required members, and
            // no sound without a name.
            let section = |key: &str, members: &[&str]| {
                m.get(key).and_then(Members::of).is_some_and(|section| {
                    members
                        .iter()
                        .all(|member| section.get(member).is_some_and(Value::is_string))
                })
            };
            // A section the parser makes nothing of goes whole; where the file has it as an
            // object, the crate notes that at the section, at the information level: the
            // game's line at its member does not say that the other members go with it.
            let left_out = |key: &str, r: &mut Reader<'_>| {
                if m.get(key).is_some_and(Value::is_object) {
                    r.key(key, |r| {
                        r.push(
                            DiagnosticCode::from(&core_codes::DROPPED_FIELD),
                            bedrock_addon_core::diagnostic::Severity::Info,
                        );
                    });
                }
            };
            if !section("particle_effect", &["effect", "type"])
                || node
                    .particle_effect
                    .as_ref()
                    .is_some_and(|effect| !written_kind(m, effect.kind))
            {
                node.particle_effect = None;
                left_out("particle_effect", r);
            }
            if !section("sound_effect", &["event_name"]) {
                node.sound_effect = None;
                left_out("sound_effect", r);
            }
        }
        node
    }
}

/// Whether `particle_effect.type` of `m` is written as `kind`: a legacy type the reader had
/// to replace is no effect.
fn written_kind(m: Members<'_>, kind: ParticleEffectType) -> bool {
    m.get("particle_effect")
        .and_then(Members::of)
        .and_then(|section| section.get("type"))
        .and_then(Value::as_string)
        .is_some_and(|text| text.as_str() == kind.as_str())
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

impl Node for EventNode {
    fn read(value: &Value, r: &mut Reader<'_>, _: Opt) -> Option<Self> {
        match Members::of(value) {
            Some(members) => Some(Self::read_node(members, false, r)),
            None => {
                r.wrong_type(value, Some(&codes::EVENT_NO_DATA));
                None
            }
        }
    }

    fn fallback(_: Opt) -> Self {
        Self::default()
    }

    fn write(&self, w: &mut Writer) -> Result<Value, EncodeError> {
        let mut out = VObject::new();
        self.write_node(&mut out, false, w)?;
        Ok(out.into())
    }

    fn walk(&mut self, era: Era, _: Opt, v: &mut MolangVisitor<'_>) {
        if let Some(sequence) = &mut self.sequence {
            walk("sequence", sequence, era, Opt::NONE, v);
        }
        if let Some(randomize) = &mut self.randomize {
            v.enter(Step::Key("randomize"));
            for (i, option) in randomize.iter_mut().enumerate() {
                v.enter(Step::Index(i));
                option.node.walk(era, Opt::NONE, v);
                v.leave();
            }
            v.leave();
        }
        if let Some(expression) = &mut self.expression {
            walk("expression", expression, era, EXPRESSION, v);
        }
        if let Some(effect) = &mut self.particle_effect {
            walk("particle_effect", effect, era, Opt::NONE, v);
        }
    }
}

impl EventNode {
    fn write_node(
        &self,
        out: &mut VObject,
        random: bool,
        w: &mut Writer,
    ) -> Result<(), EncodeError> {
        // A node this deep is in no text the loader reads (and the walk down to it would
        // not end on a chain a caller built without end).
        if w.depth() >= MAX_JSON_DEPTH {
            return Err(w.refuse(Unrepresentable::Unsupported));
        }
        put_opt(out, "sequence", self.sequence.as_ref(), w)?;
        if let Some(randomize) = &self.randomize {
            let options = w.key("randomize", |w| {
                let mut options = VArray::new();
                for (i, option) in randomize.iter().enumerate() {
                    options.push(w.index(i, |w| option.write(w))?);
                }
                Ok(options)
            })?;
            out.insert("randomize", options);
        }
        put_opt(out, "expression", self.expression.as_ref(), w)?;
        put_opt(out, "particle_effect", self.particle_effect.as_ref(), w)?;
        put_opt(out, "sound_effect", self.sound_effect.as_ref(), w)?;
        put_opt(out, "log", self.log.as_ref(), w)?;
        let known = if random {
            Self::known_in_randomize
        } else {
            Self::known
        };
        w.extra(out, &self.extra, known)
    }
}

impl Node for RandomEventNode {
    // The legacy parser skips an option without a number for a weight, or with a negative
    // one; a 1.20.80 document requires the member and has no bound.
    fn read(value: &Value, r: &mut Reader<'_>, _: Opt) -> Option<Self> {
        let Some(members) = Members::of(value) else {
            r.wrong_type(value, Some(&codes::EVENT_NO_DATA));
            return None;
        };
        let weight = match members.get("weight") {
            Some(weight) => r.key("weight", |r| {
                let read = float(weight, r);
                if read.is_none() {
                    r.wrong_type(weight, Some(&codes::WEIGHT));
                }
                read
            }),
            None => {
                r.missing(members, "weight", Some(&codes::WEIGHT));
                None
            }
        };
        if r.legacy() {
            let negative = weight.is_some_and(|weight| weight < 0.0);
            if negative {
                r.key("weight", |r| r.warn(&codes::WEIGHT_NEGATIVE));
            }
            if weight.is_none() || negative {
                // The option is skipped with every member it has; the line is at its
                // weight alone. The crate notes the skip at the option, at the information
                // level.
                r.push(
                    DiagnosticCode::from(&core_codes::DROPPED_FIELD),
                    Severity::Info,
                );
                return None;
            }
        }
        Some(Self {
            weight: weight.unwrap_or_default(),
            node: EventNode::read_node(members, true, r),
        })
    }

    fn fallback(_: Opt) -> Self {
        Self::default()
    }

    fn write(&self, w: &mut Writer) -> Result<Value, EncodeError> {
        let mut out = VObject::new();
        let weight = w.key("weight", |w| self.weight.write(w))?;
        out.insert("weight", weight);
        self.node.write_node(&mut out, true, w)?;
        Ok(out.into())
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
