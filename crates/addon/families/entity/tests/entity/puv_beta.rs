//! The game's beta-load test of 1.26.60 (fixture `puv_load-1.json`): the `move_indoors` actor
//! of its test helper at `beta`.

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::versioning::Loaded;

use super::common::{cx, fixture, outcome};

fn beta_actor() -> String {
    fixture("puv_beta/move_indoors_beta.json")
}

/// `BetaActorJson_NonBasePackWithoutBetaApis_FailsToLoad`: invalid, exactly one parse entry,
/// the Puv text.
#[test]
fn a_beta_actor_of_a_non_base_pack_without_beta_apis_fails_with_one_entry() {
    let refused = outcome(&beta_actor(), &cx());
    assert!(refused.value.is_none());
    let entries: Vec<_> = refused.diagnostics.iter().collect();
    assert_eq!(entries.len(), 1, "{entries:#?}");
    assert_eq!(entries[0].severity, Severity::Error);
    assert_eq!(
        entries[0].code.to_string(),
        "Using a beta format_version is only allowed for base packs or when the BetaApis toggle is enabled."
    );
}

/// `BetaActorJson_BasePackWithoutBetaApis_Loads`, `…_BasePackWithBetaApis_Loads`,
/// `…_NonBasePackWithBetaApis_Loads`: the beta document loads.
#[test]
fn a_beta_actor_loads_for_a_base_pack_or_with_beta_apis() {
    for (base_game, beta_apis) in [(true, false), (true, true), (false, true)] {
        let mut context = cx();
        context.is_base_game_pack = base_game;
        context.beta_apis = beta_apis;
        let loaded = outcome(&beta_actor(), &context);
        assert!(
            matches!(loaded.value, Some(Loaded::Beta(_))),
            "base {base_game}, beta APIs {beta_apis}: {:#?}",
            loaded.diagnostics
        );
    }
}
