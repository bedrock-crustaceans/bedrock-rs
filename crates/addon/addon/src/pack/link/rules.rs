//! `rules!`, the one way a link group lists its rule files and the order their entries run.
//!
//! A group writes each of its modules once, with what it needs, and then its runs, each a
//! function calling entries `module::function` in the order written:
//!
//! ```text
//! rules! {
//!     mod blocks_json needs ["blocks_json", any["block", "sounds_json"]];
//!     mod told needs any[["block", "blocks_json"], ["blocks_json", "sounds_json"]];
//!     mod client_biome_biome needs ["biome", "client_biome"];
//!     /// The rules of the client biomes.
//!     run biomes() = client_biome_biome::run, client_biome_biome::water + ["biomes_client"];
//!     run checks(users: &[User<'_>]) for user in users = user_geometry::check;
//! }
//! ```
//!
//! A list in brackets needs every one of its members, `any[..]` one of them, and a member is
//! a feature name, an `any[..]` of feature names, or (as a member of `any[..]`) a list of
//! feature names. A module is compiled under its needs, and an entry under its module's needs
//! and the features after its `+`, so a rule's features are written once, on its module. An
//! entry calls `module::function(cx, out, <the run's parameters>)`; in a run with
//! `for x in xs` it calls `module::function(cx, out, x)` for every `x` of `xs`, entry by
//! entry. The runs are compiled whatever is on; with no module of an entry they call nothing.
//!
//! Rules that share a state built once per pass sit in a `with` block, after the modules and
//! before the runs:
//!
//! ```text
//! with entities = walk::Entities::new(cx) needs ["entity"] {
//!     mod entity_block needs ["block"];
//!     mod entity_entity needs [];
//!     /// The rules over the entities' references.
//!     run references = entity_block::run, entity_entity::run;
//! }
//! ```
//!
//! Its first path segment, `walk`, is a module compiled under the block's needs. Every module
//! of the block needs those too, on top of its own (`entity_block` needs `entity` and
//! `block`). The block's one run is a function of `cx` and `out` alone. Under the block's
//! needs it builds the state with the path called on `cx` and calls each entry as
//! `module::function(cx, out, &entities)`, in the order written. The block's own needs are a
//! plain list of at least one feature (no `any[..]`, not empty), and the block holds one run,
//! written without a parameter list.
//!
//! For each module the macro also defines a macro of the module's name: `name! { .. }` is
//! its argument under the module's needs, for a group that calls a rule by hand. Under
//! `cfg(test)` it emits `NEEDS`, every module with the features its needs name.

/// The modules and runs of a link group: see the module documentation.
macro_rules! rules {
    (
        $(mod $name:ident needs $($any:ident)? [$($needs:tt)*];)*
        $(
            with $state:ident = $home:ident $(:: $make:ident)+ ($state_cx:ident)
                needs $state_needs:tt {
                $(mod $with_name:ident needs $($with_any:ident)? $with_needs:tt;)*
                $(#[$with_attr:meta])*
                run $with_vis:vis $with_run:ident =
                    $($with_module:ident::$with_entry:ident
                        $(+ [$($with_extra:literal),+ $(,)?])?),+ $(,)?;
            }
        )*
        $(
            $(#[$attr:meta])*
            run $vis:vis $run:ident($($param:ident: $type:ty),* $(,)?)
                $(for $each:ident in $list:ident)? =
                $($module:ident::$entry:ident $(+ [$($extra:literal),+ $(,)?])?),+ $(,)?;
        )*
    ) => {
        $(rule_module! { ($) $name $($any)? [$($needs)*] })*
        $(
            rule_module! { ($) @with $state_needs $home [] }
            $(rule_module! { ($) @with $state_needs $with_name $($with_any)? $with_needs })*
            rule_state_run! {
                [$(#[$with_attr])*] $with_vis $with_run [$state_cx out]
                $state ($home $(:: $make)+)
                [$($with_module::$with_entry [$($($with_extra),+)?])+]
            }
        )*
        $(
            rule_run! {
                [$(#[$attr])*] $vis $run [cx out] [$($param: $type),*] (cx, out $(, $param)*)
                [$(for $each in $list)?] [$($module::$entry [$($($extra),+)?])+]
            }
        )*
        /// Every module of the group with the features its needs name.
        #[cfg(test)]
        pub(super) const NEEDS: &[(&str, &[&str])] = &[
            $((stringify!($name), $name!(@needs)),)*
            $(
                (stringify!($home), $home!(@needs)),
                $((stringify!($with_name), $with_name!(@needs)),)*
            )*
        ];
    };
}

/// One module of [`rules!`]: `mod name;` under the module's needs and the macro `name!`.
/// The needs are turned into a `cfg` predicate a member at a time, the feature names
/// collected beside it.
macro_rules! rule_module {
    (($d:tt) @with [$($state:tt)*] $name:ident any [$($alternatives:tt)*]) => {
        rule_module! { ($d) $name [$($state)*, any[$($alternatives)*]] }
    };
    (($d:tt) @with [$($state:tt)*] $name:ident [$($members:tt)*]) => {
        rule_module! { ($d) $name [$($state)*, $($members)*] }
    };
    (($d:tt) $name:ident any [$($alternatives:tt)*]) => {
        rule_module! { @any $d $name [$($alternatives)*] }
    };
    (($d:tt) $name:ident [$($members:tt)*]) => {
        rule_module! { @all $d $name [] [] [$($members)*] }
    };
    (@any $d:tt $name:ident [$($feature:literal),+ $(,)?]) => {
        rule_module! { @emit $d $name [any($(feature = $feature),+)] [$($feature,)+] }
    };
    (@any $d:tt $name:ident [$([$($feature:literal),+ $(,)?]),+ $(,)?]) => {
        rule_module! {
            @emit $d $name [any($(all($(feature = $feature),+)),+)] [$($($feature,)+)+]
        }
    };
    (@all $d:tt $name:ident [$($cfg:tt)*] [$($names:tt)*] []) => {
        rule_module! { @emit $d $name [all($($cfg)*)] [$($names)*] }
    };
    (@all $d:tt $name:ident [$($cfg:tt)*] [$($names:tt)*]
        [$feature:literal $(, $($rest:tt)*)?]) => {
        rule_module! {
            @all $d $name [$($cfg)* feature = $feature,] [$($names)* $feature,] [$($($rest)*)?]
        }
    };
    (@all $d:tt $name:ident [$($cfg:tt)*] [$($names:tt)*]
        [any [$($feature:literal),+ $(,)?] $(, $($rest:tt)*)?]) => {
        rule_module! {
            @all $d $name [$($cfg)* any($(feature = $feature),+),] [$($names)* $($feature,)+]
            [$($($rest)*)?]
        }
    };
    (@all $d:tt $name:ident [$($cfg:tt)*] [$($names:tt)*]
        [any [$([$($feature:literal),+ $(,)?]),+ $(,)?] $(, $($rest:tt)*)?]) => {
        rule_module! {
            @all $d $name [$($cfg)* any($(all($(feature = $feature),+)),+),]
            [$($names)* $($($feature,)+)+] [$($($rest)*)?]
        }
    };
    (@emit $d:tt $name:ident [$cfg:meta] [$($names:tt)*]) => {
        #[cfg($cfg)]
        mod $name;

        #[allow(unused_macros, reason = "a helper module has no entry and no call by hand")]
        macro_rules! $name {
            (@needs) => {
                &[$($names)*]
            };
            ($d($d body:tt)*) => {
                #[cfg($cfg)]
                {
                    $d($d body)*
                }
            };
        }
    };
}

/// One run of [`rules!`]: a function calling its entries in order, each under its module's
/// needs and its own.
macro_rules! rule_run {
    (
        [$(#[$attr:meta])*] $vis:vis $run:ident [$cx:ident $out:ident]
        [$($param:ident: $type:ty),*] $arguments:tt []
        [$($module:ident::$entry:ident [$($extra:literal),*])+]
    ) => {
        $(#[$attr])*
        $vis fn $run(
            $cx: &$crate::pack::link::LinkCx<'_>,
            $out: &mut $crate::pack::link::Findings
            $(, $param: $type)*
        ) {
            let _ = ($cx, &*$out $(, $param)*);
            $(
                $module! {
                    #[cfg(all($(feature = $extra),*))]
                    $module::$entry $arguments;
                }
            )+
        }
    };
    (
        [$(#[$attr:meta])*] $vis:vis $run:ident [$cx:ident $out:ident]
        [$($param:ident: $type:ty),*] $arguments:tt [for $each:ident in $list:ident]
        [$($module:ident::$entry:ident [$($extra:literal),*])+]
    ) => {
        $(#[$attr])*
        $vis fn $run(
            $cx: &$crate::pack::link::LinkCx<'_>,
            $out: &mut $crate::pack::link::Findings
            $(, $param: $type)*
        ) {
            let _ = ($cx, &*$out $(, $param)*);
            $(
                $module! {
                    #[cfg(all($(feature = $extra),*))]
                    for $each in $list {
                        $module::$entry($cx, $out, $each);
                    }
                }
            )+
        }
    };
}

/// The run of a `with` block of [`rules!`]: a function that builds the state once, under the
/// needs of the state's module, and calls its entries with it in order, each under its
/// module's needs and its own.
macro_rules! rule_state_run {
    (
        [$(#[$attr:meta])*] $vis:vis $run:ident [$cx:ident $out:ident]
        $state:ident ($home:ident $(:: $make:ident)+)
        [$($module:ident::$entry:ident [$($extra:literal),*])+]
    ) => {
        $(#[$attr])*
        $vis fn $run(
            $cx: &$crate::pack::link::LinkCx<'_>,
            $out: &mut $crate::pack::link::Findings,
        ) {
            let _ = ($cx, &*$out);
            $home! {
                let $state = $home $(:: $make)+($cx);
                let _ = &$state;
                $(
                    $module! {
                        #[cfg(all($(feature = $extra),*))]
                        $module::$entry($cx, $out, &$state);
                    }
                )+
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    /// A group's `NEEDS`: every module with the features its needs name.
    type Needs = &'static [(&'static str, &'static [&'static str])];

    /// The groups whose rules are listed with `rules!`, each with the needs table the macro
    /// emits for it. A hand list: a new group joins it here, or its needs go unchecked.
    const GROUPS: &[(&str, Needs)] = &[
        ("content", crate::pack::link::content::NEEDS),
        ("entity", crate::pack::link::entity::NEEDS),
        ("resource", crate::pack::link::resource::NEEDS),
        ("visual", crate::pack::link::visual::NEEDS),
        ("world", crate::pack::link::world::NEEDS),
    ];

    /// The words of a module name that are no feature of the crate, each with the features
    /// it may stand for. In `visual` an `entity` is a client entity and an `item` an
    /// attachable, so those two words count the client families as well.
    const WORDS: &[(&str, &[&str])] = &[
        (
            "entity",
            &["entity", "client_entity", "atomic_client_entity"],
        ),
        ("item", &["item", "attachable"]),
        (
            "render",
            &["render_controller", "material", "deferred_settings"],
        ),
        ("sound", &["sound_definitions", "sounds_json", "sound_file"]),
        ("text", &["lang", "languages"]),
        ("texture", &["texture_atlas", "texture_file", "texture_set"]),
        ("ui", &["font", "loading_messages"]),
        (
            "user",
            &["client_entity", "atomic_client_entity", "attachable"],
        ),
    ];

    /// The modules named for what they do rather than for the modules they relate.
    const HELPERS: &[&str] = &["told", "walk"];

    /// The keys of the `[features]` table of the crate's `Cargo.toml`.
    fn features() -> BTreeSet<&'static str> {
        let manifest = include_str!("../../../Cargo.toml");
        manifest
            .lines()
            .skip_while(|line| *line != "[features]")
            .skip(1)
            .take_while(|line| !line.starts_with('['))
            .filter(|line| line.starts_with(|c: char| c.is_ascii_alphanumeric()))
            .filter_map(|line| line.split_once(" = ").map(|(key, _)| key))
            .collect()
    }

    /// The words of `module`, each the longest run of its `_`-separated parts that is a
    /// feature or one of [`WORDS`]; a part that starts no such run is an `Err`.
    fn words<'m>(module: &'m str, features: &BTreeSet<&str>) -> Vec<Result<String, &'m str>> {
        let parts: Vec<&str> = module.split('_').collect();
        let known =
            |word: &str| features.contains(word) || WORDS.iter().any(|(known, _)| *known == word);
        let mut words = Vec::new();
        let mut start = 0;
        while start < parts.len() {
            if let Some((end, word)) = (start + 1..=parts.len())
                .rev()
                .map(|end| (end, parts[start..end].join("_")))
                .find(|(_, word)| known(word))
            {
                words.push(Ok(word));
                start = end;
            } else {
                words.push(Err(parts[start]));
                start += 1;
            }
        }
        words
    }

    /// Every need of a rule module is a feature of the crate, and every word of its name is
    /// among its needs: `<a>_<b>.rs` is compiled with `a` and `b` on, so a rule that names a
    /// module runs only when that module is there.
    #[test]
    fn every_rule_needs_only_features_the_crate_has_and_its_name_says() {
        let features = features();
        assert!(features.len() > 50, "found {} features", features.len());
        let mut wrong = Vec::new();
        for (group, modules) in GROUPS {
            for (module, needs) in *modules {
                for need in *needs {
                    if !features.contains(need) {
                        wrong.push(format!("{group}::{module} needs `{need}`, no feature"));
                    }
                }
                if HELPERS.contains(module) {
                    continue;
                }
                for word in words(module, &features) {
                    let word = match word {
                        Ok(word) => word,
                        Err(part) => {
                            wrong.push(format!("{group}::{module}: `{part}` names nothing"));
                            continue;
                        }
                    };
                    let named = match WORDS.iter().find(|(known, _)| *known == word) {
                        Some((_, features)) => features.iter().any(|f| needs.contains(f)),
                        None => needs.contains(&word.as_str()),
                    };
                    if !named {
                        wrong.push(format!("{group}::{module} does not need `{word}`"));
                    }
                }
            }
        }
        assert_eq!(wrong, [""; 0], "rule modules whose needs and name disagree");
    }
}
