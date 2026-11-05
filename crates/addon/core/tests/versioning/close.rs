//! `Family::close`, the post-parse hook every family may implement, a family with a beta
//! struct included; `FileFamily::close` delegates to it unless the one-file family says
//! otherwise.

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, Location, Phase, Severity, VanillaText,
};
use bedrock_addon_core::json::JsonSlice;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{ComponentTable, Family};

use crate::toy::{Toy120, ToyBeta, ToyComponent, v1_0_0};

/// A family with a real beta struct that closes a failed load with a line of its own.
pub struct Closing;

pub mod closing {
    use super::*;
    bedrock_addon_core::version_chain! {
        family = Closing;
        /// The toy's chain for the closing family.
        pub enum Any {
            V1_0_0(v1_0_0::Toy),
            V1_1_0(crate::toy::v1_1_0::Toy),
            V1_2_0(Toy120),
        }
        beta = Beta(ToyBeta);
    }
}

const CLOSING_LINE: &str = "Failed to load toy";

impl Family for Closing {
    const NAME: &'static str = "toy";
    const PAYLOAD_KEY: &'static str = "minecraft:toy";
    const DIRECTORIES: &'static [&'static str] = &["toys"];
    const FLOOR: FormatVersion = FormatVersion::new(1, 0, 0);
    type Latest = Toy120;
    type Beta = ToyBeta;
    type Component = ToyComponent;
    type Any = closing::Any;
    fn components() -> &'static ComponentTable<Self> {
        &ComponentTable::EMPTY
    }

    fn close(diagnostics: &mut Diagnostics, _: Option<JsonSlice<'_>>, _: &PackPath, failed: bool) {
        if failed {
            diagnostics.push(Diagnostic::new(
                DiagnosticCode::DocumentRejected {
                    text: Some(VanillaText::Fixed(CLOSING_LINE)),
                },
                Severity::Error,
                Phase::Parse,
                Location::default(),
            ));
        }
    }
}

fn closed<F: Family>(failed: bool) -> Diagnostics {
    let mut diagnostics = Diagnostics::default();
    F::close(
        &mut diagnostics,
        None,
        &PackPath::from_static("toys/a.json"),
        failed,
    );
    diagnostics
}

#[test]
fn a_family_with_a_beta_struct_closes_a_failed_load() {
    let failed = closed::<Closing>(true);
    assert_eq!(failed.len(), 1, "the failed load gets the closing line");
    assert!(matches!(
        failed.iter().next().map(|d| &d.code),
        Some(DiagnosticCode::DocumentRejected {
            text: Some(VanillaText::Fixed(CLOSING_LINE))
        })
    ));
    assert_eq!(
        closed::<Closing>(false).len(),
        0,
        "a loaded file is left alone"
    );
}

#[test]
fn close_is_a_no_op_by_default() {
    assert_eq!(closed::<crate::toy::Toy>(true).len(), 0);
}
