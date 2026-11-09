//! The committed command tables against the generator's list of built-in parameter types.

use bedrock_addon_core::vanilla;

#[test]
fn every_parameter_type_is_an_enum_or_a_built_in_token() {
    // An upper-cased enum name or one of the 26 tokens; every enum is referenced.
    let tokens = super::step::COMMAND_TOKENS;
    let mut referenced = std::collections::BTreeSet::new();
    for param in vanilla::commands()
        .iter()
        .flat_map(|c| c.overloads)
        .flat_map(|o| o.params)
    {
        match vanilla::command_enum(param.type_name) {
            Some(found) => {
                referenced.insert(found.name);
            }
            None => assert!(tokens.contains(&param.type_name), "{}", param.type_name),
        }
    }
    assert_eq!(referenced.len(), vanilla::command_enums().len());
    let aim = vanilla::command_enum("AIMASSISTACTIONSET").unwrap();
    assert_eq!(aim.name, "AimAssistActionSet");
    assert_eq!(aim.values, ["set"]);
}
