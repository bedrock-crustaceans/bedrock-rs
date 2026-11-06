//! `Legacy<P, A, B, C>` and `LegacyStep`: a lamp family whose two older eras are one payload
//! type, a real step, an identity step (`retag`), and a newest struct held boxed (the `Box`
//! impls of `Versioned` and `VersionStruct`).

use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput};
use bedrock_addon_core::versioning::{
    Family, Legacy, LegacyPayload, LegacyStep, Loaded, NoComponents, ParseCx, Upgrade, UpgradeCx,
    VersionChain, VersionStruct, Versioned, parse_payload, write_payload,
};
use facet::Facet;

use crate::envelope::load_with;

const V100: FormatVersion = FormatVersion::new(1, 0, 0);
const V110: FormatVersion = FormatVersion::new(1, 1, 0);
const V120: FormatVersion = FormatVersion::new(1, 2, 0);
const V130: FormatVersion = FormatVersion::new(1, 3, 0);

/// The lamp family.
pub struct Lamp;

/// The payload of every older era: `level`, and `glow` from 1.1.0.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct LampPayload {
    #[facet(default)]
    pub level: u8,
    #[facet(default)]
    pub glow: Option<u8>,
}
bedrock_addon_core::impl_no_molang!(LampPayload);

impl LegacyPayload for LampPayload {
    fn parse(
        payload: JsonSlice<'_>,
        version: FormatVersion,
        cx: &ParseCx<'_>,
        d: &mut Diagnostics,
    ) -> Option<Self> {
        let mut lamp: Self = parse_payload(payload, cx, d)?;
        if version < V110 {
            // 1.0.0 has no `glow`.
            lamp.glow = None;
        }
        Some(lamp)
    }

    fn write(&self, version: FormatVersion) -> Result<Value, EncodeError> {
        let mut lamp = self.clone();
        if version < V110 {
            lamp.glow = None;
        }
        write_payload(&lamp)
    }
}

/// The newest struct, 1.3.0.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct LampDoc {
    pub brightness: u8,
}
bedrock_addon_core::impl_no_molang!(LampDoc);

impl Versioned for LampDoc {
    const VERSION: FormatVersion = V130;
}

impl VersionStruct for LampDoc {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        parse_payload(payload, cx, d)
    }

    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(self)
    }
}

pub mod history {
    use super::{Lamp, LampDoc, LampPayload, Legacy};

    pub type V1_0_0 = Legacy<LampPayload, 1, 0, 0>;
    pub type V1_1_0 = Legacy<LampPayload, 1, 1, 0>;
    pub type V1_2_0 = Legacy<LampPayload, 1, 2, 0>;

    bedrock_addon_core::version_chain! {
        family = Lamp;
        /// The struct the file's `format_version` selected.
        #[derive(Clone, Debug, PartialEq)]
        pub enum LampHistory {
            V1_0_0(V1_0_0),
            V1_1_0(V1_1_0),
            V1_2_0(V1_2_0),
            V1_3_0(Box<LampDoc>),
        }
    }
}

/// 1.0.0 → 1.1.0: `glow` is the level.
impl LegacyStep<1, 0, 0> for LampPayload {
    type Next = history::V1_1_0;

    fn step(this: history::V1_0_0, _: &mut UpgradeCx<'_>) -> history::V1_1_0 {
        let mut lamp = this.0;
        lamp.glow = Some(lamp.level);
        Legacy(lamp)
    }
}

/// 1.1.0 → 1.2.0: the identity step.
impl LegacyStep<1, 1, 0> for LampPayload {
    type Next = history::V1_2_0;

    fn step(this: history::V1_1_0, _: &mut UpgradeCx<'_>) -> history::V1_2_0 {
        this.retag()
    }
}

/// 1.2.0 → 1.3.0: into the newest struct, boxed.
impl LegacyStep<1, 2, 0> for LampPayload {
    type Next = Box<LampDoc>;

    fn step(this: history::V1_2_0, _: &mut UpgradeCx<'_>) -> Box<LampDoc> {
        Box::new(LampDoc {
            brightness: this.0.level.saturating_add(this.0.glow.unwrap_or(0)),
        })
    }
}

impl Family for Lamp {
    const NAME: &'static str = "lamp";
    const PAYLOAD_KEY: &'static str = "demo:lamp";
    const FLOOR: FormatVersion = V100;
    type Latest = Box<LampDoc>;
    type Beta = std::convert::Infallible;
    type Component = NoComponents;
    type History = history::LampHistory;
}

fn document(version: &str, payload: &str) -> String {
    format!(r#"{{"format_version": "{version}", "demo:lamp": {payload}}}"#)
}

fn parse_at(version: FormatVersion, payload: &str) -> (Option<history::LampHistory>, Diagnostics) {
    let mut d = Diagnostics::default();
    let any = <history::LampHistory as VersionChain<Lamp>>::parse(
        JsonSlice::new(payload),
        version,
        &LoadContext::detached(),
        &mut d,
    );
    (any, d)
}

#[test]
fn a_legacy_chain_lists_the_eras_and_the_boxed_newest() {
    assert_eq!(
        <history::LampHistory as VersionChain<Lamp>>::VERSIONS,
        [V100, V110, V120, V130]
    );
    assert_eq!(<history::V1_1_0 as Versioned>::VERSION, V110);
    assert_eq!(<Box<LampDoc> as Versioned>::VERSION, V130);
}

#[test]
fn a_floor_document_walks_the_legacy_steps_into_the_boxed_newest() {
    let outcome = load_with::<Lamp>(
        &document("1.0.0", r#"{"level": 3}"#),
        &LoadContext::detached(),
    );
    let Some(Loaded::Stable(lamp)) = &outcome.value else {
        panic!("{:?}", outcome.diagnostics);
    };
    assert_eq!(
        **lamp,
        LampDoc { brightness: 6 },
        "the step's glow, then the sum"
    );
    assert!(outcome.version.upgraded);
    assert!(outcome.diagnostics.is_empty(), "{:?}", outcome.diagnostics);
}

#[test]
fn an_identity_step_hands_the_payload_on() {
    let payload = LampPayload {
        level: 2,
        glow: Some(5),
    };
    let next = Legacy::<LampPayload, 1, 1, 0>(payload.clone()).retag::<1, 2, 0>();
    assert_eq!(next.0, payload);
    assert_eq!(next.level, 2, "the struct derefs to its payload");
    let mut d = Diagnostics::default();
    let mut cx = UpgradeCx {
        load: &LoadContext::detached(),
        original_version: V110,
        identifier: "demo:x",
        actor_type: bedrock_addon_core::vanilla::ActorType::Mob,
        skip_section_a: false,
        diagnostics: &mut d,
    };
    let stepped: history::V1_2_0 = Legacy::<LampPayload, 1, 1, 0>(payload.clone()).upgrade(&mut cx);
    assert_eq!(stepped, Legacy(payload));
}

#[test]
fn an_era_reads_and_writes_by_its_own_version() {
    let (old, d) = parse_at(V100, r#"{"level": 1, "glow": 9}"#);
    assert!(d.is_empty(), "{d:?}");
    let old = old.expect("parsed");
    assert_eq!(
        old,
        history::LampHistory::V1_0_0(Legacy(LampPayload {
            level: 1,
            glow: None
        }))
    );
    assert_eq!(
        old.to_json().expect("written"),
        "{\n  \"format_version\": \"1.0.0\",\n  \"demo:lamp\": {\n    \"level\": 1\n  }\n}\n"
    );
    let (newer, _) = parse_at(V110, r#"{"level": 1, "glow": 9}"#);
    let newer = newer.expect("parsed");
    assert_eq!(newer.parsed_version(), V110);
    assert_eq!(
        newer.to_json().expect("written"),
        "{\n  \"format_version\": \"1.1.0\",\n  \"demo:lamp\": {\n    \"level\": 1,\n    \"glow\": 9\n  }\n}\n"
    );
}

#[test]
fn a_boxed_struct_reads_and_writes_as_its_struct() {
    let text = r#"{"brightness": 4, "extra": true}"#;
    let cx = ParseCx {
        load: &LoadContext::detached(),
        written: FormatVersionInput {
            version: Some(V130),
            beta_literal: false,
            prerelease: false,
        },
        original_version: V130,
        at: bedrock_addon_core::diagnostic::JsonPath::new().child("demo:lamp"),
    };
    let (mut plain_d, mut boxed_d) = (Diagnostics::default(), Diagnostics::default());
    let plain = LampDoc::parse(JsonSlice::new(text), &cx, &mut plain_d);
    let boxed = <Box<LampDoc>>::parse(JsonSlice::new(text), &cx, &mut boxed_d);
    assert_eq!(boxed.as_deref(), plain.as_ref());
    assert_eq!(
        boxed_d, plain_d,
        "the same findings, the unknown member's among them"
    );
    assert!(!plain_d.is_empty());
    let (plain, boxed) = (plain.expect("parsed"), boxed.expect("parsed"));
    assert_eq!(boxed.write().ok(), plain.write().ok());
    let (newest, _) = parse_at(V130, r#"{"brightness": 4}"#);
    assert_eq!(
        newest.expect("parsed").to_json().expect("written"),
        "{\n  \"format_version\": \"1.3.0\",\n  \"demo:lamp\": {\n    \"brightness\": 4\n  }\n}\n"
    );
}
