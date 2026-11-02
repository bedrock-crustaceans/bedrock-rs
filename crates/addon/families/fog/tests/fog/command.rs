//! `k01`–`k06`: the server's fog
//! stack behind `/fog`.

use bedrock_addon_family_fog::{
    FOG_STACK_LIMIT, FogCommandError, FogStack, is_valid_fog_identifier,
};

fn pairs(stack: &FogStack) -> Vec<(&str, &str)> {
    stack
        .iter()
        .map(|(fog, user)| (fog.as_str(), user.as_str()))
        .collect()
}

#[test]
fn k01_push_invalid_id() {
    let mut stack = FogStack::new();
    for id in ["Bad!Id", "noname", ":x", "a:", "a:b:c", "a b:c", ""] {
        let error = stack.push(id, "u1").unwrap_err();
        assert_eq!(error, FogCommandError::InvalidFogId, "{id}");
        assert_eq!(error.lang_key(), Some("commands.fog.invalidFogId"));
    }
    assert!(stack.is_empty());
    // The check is on the lower-cased identifier; it is stored as given.
    stack.push("Demo:Fog_1.x", "u1").unwrap();
    assert_eq!(pairs(&stack), [("Demo:Fog_1.x", "u1")]);
    assert!(is_valid_fog_identifier("minecraft:fog_default"));
    assert!(!is_valid_fog_identifier("minecraft"));
}

#[test]
fn k02_push_limit() {
    let mut stack = FogStack::new();
    assert_eq!(FOG_STACK_LIMIT, 16);
    for n in 0..16 {
        stack.push(&format!("a:f{n}"), "u").unwrap();
    }
    let error = stack.push("a:f16", "u").unwrap_err();
    assert_eq!(error, FogCommandError::LimitReached);
    assert_eq!(error.lang_key(), Some("commands.fog.limitReached"));
    assert_eq!(stack.len(), 16);
    // An invalid identifier is reported before the limit.
    assert_eq!(stack.push("bad", "u"), Err(FogCommandError::InvalidFogId));
    // A pop makes room again.
    stack.pop("u").unwrap();
    stack.push("a:f16", "u").unwrap();
    assert_eq!(stack.fog_ids().last(), Some(&"a:f16"));
}

#[test]
fn k03_pop_top_most() {
    let mut stack = FogStack::new();
    stack.push("a:x", "u").unwrap();
    stack.push("a:y", "u").unwrap();
    stack.push("a:z", "v").unwrap();
    stack.pop("u").unwrap();
    assert_eq!(pairs(&stack), [("a:x", "u"), ("a:z", "v")]);
    stack.pop("v").unwrap();
    assert_eq!(pairs(&stack), [("a:x", "u")]);
    // The packet lists the fog identifiers, bottom to top.
    assert_eq!(stack.fog_ids(), ["a:x"]);
}

#[test]
fn k04_remove_all() {
    let mut stack = FogStack::new();
    stack.push("a:x", "u").unwrap();
    stack.push("a:y", "v").unwrap();
    stack.push("a:z", "u").unwrap();
    stack.remove("u").unwrap();
    assert_eq!(pairs(&stack), [("a:y", "v")]);
    assert_eq!(stack.fog_ids(), ["a:y"]);
}

#[test]
fn k05_pop_unknown() {
    let mut stack = FogStack::new();
    stack.push("a:x", "u").unwrap();
    for error in [
        stack.pop("nope").unwrap_err(),
        stack.remove("nope").unwrap_err(),
    ] {
        assert_eq!(error, FogCommandError::InvalidUserId);
        assert_eq!(error.lang_key(), Some("commands.fog.invalidUserId"));
    }
    // The user id is compared as written.
    assert_eq!(stack.pop("U"), Err(FogCommandError::InvalidUserId));
    assert_eq!(pairs(&stack), [("a:x", "u")]);
}

#[test]
fn k06_nbt_roundtrip() {
    let saved = ["a:x", "u", "a:y", "v"];
    let stack = FogStack::from_saved(&saved).unwrap();
    assert_eq!(pairs(&stack), [("a:x", "u"), ("a:y", "v")]);
    assert_eq!(stack.to_saved(), saved);
    assert_eq!(FogStack::from_saved(&stack.to_saved()).unwrap(), stack);

    let odd = FogStack::from_saved(&["a:x", "u", "a:y"]).unwrap_err();
    assert_eq!(odd, FogCommandError::OddSavedStack);
    assert_eq!(odd.lang_key(), None);
    // A pair whose fog identifier is not valid is left out.
    let stack = FogStack::from_saved(&["Bad!Id", "u", "a:y", "v"]).unwrap();
    assert_eq!(pairs(&stack), [("a:y", "v")]);
    assert!(FogStack::from_saved::<&str>(&[]).unwrap().is_empty());
}

#[test]
fn iter_walks_the_entries_bottom_first() {
    let mut stack = FogStack::new();
    stack.push("a:x", "u1").unwrap();
    stack.push("b:y", "u2").unwrap();
    let walked: Vec<(&str, &str)> = stack
        .iter()
        .map(|(fog, user)| (fog.as_str(), user.as_str()))
        .collect();
    assert_eq!(walked, [("a:x", "u1"), ("b:y", "u2")]);
    assert_eq!(stack.len(), 2);
    assert_eq!((&stack).into_iter().count(), 2);
}
