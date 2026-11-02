//! The stack of a typed read.
//!
//! facet reads a typed target by a call of itself for each level of the type, and for each
//! level of the text where the type holds itself or an untyped member. In a debug build a
//! level takes tens of kilobytes (with facet 0.50.0-rc.7: 13 KiB below an untyped
//! member, 21 for a struct that holds itself in a list, 30 in a map, 55 in a box, 87 where
//! each level is an untagged enum, a box and a struct; an optimised build takes 2 to 11), so
//! a text within `MAX_JSON_DEPTH` took more than a megabyte, and more than the eight of a
//! main thread where the type is unkind. A stack that overflows is no error a caller can
//! handle: the process ends.
//!
//! So the door asks how deep the text nests, which its scan knows before facet reads a byte,
//! and gives the read the stack that depth can take. Two sizes, both by the depth:
//!
//! - what the thread must have left for the read to run where it is ([`to_stay`]): two to
//!   three times what the costliest type of the crates takes, in the build at hand;
//! - the stack made for the read where the thread has less ([`made`]): several times that
//!   again, since it costs address space and nothing else. It is mapped for the read with a
//!   guard page below it and unmapped after, on the same thread (no value crosses a thread,
//!   so no target need be `Send`).
//!
//! What stays outside the rule:
//!
//! - WebAssembly, where a module has the one stack it was linked with;
//! - a target on which `stacker` has no way to give a call another stack: it then runs the
//!   read where it is, without a word;
//! - a host that cannot map the stack (an address-space limit, strict overcommit, a 32-bit
//!   address space that is full): `stacker` panics there. The stack made for a text at the
//!   limit is 65 MiB of address space; pages are taken as they are touched.

/// Whether the crate is built without optimisation, as far as it can tell: with debug
/// assertions (the `dev` and `test` profiles), or at `opt-level` 0 (`build.rs`). facet is
/// built by the same profile unless a package override says otherwise; an override that
/// leaves facet unoptimised below an optimised core is the one case this misjudges, and
/// there the stack to stay on is below what the costliest type of the crates takes at the
/// limit (4.5 MiB asked for, against some 7 MiB: 128 levels of `RawTypeDesc` at 55 KiB a
/// level): such a build is short on a thread that has between the two left. No profile of
/// the workspace is one.
const UNOPTIMISED: bool = cfg!(debug_assertions) || cfg!(unoptimised);

/// The stack made for one level of a text: over five times what the costliest shape
/// takes in a debug build (87 KiB: an untagged enum, a box and a struct).
const MADE_A_LEVEL: usize = 512 * 1024;

/// The stack made for a read whatever the text: the levels a type has of its own above the
/// first array or object, and what a conversion at a leaf runs (a Molang expression is
/// compiled there, some 384 KiB unoptimised).
const MADE_FOR_A_READ: usize = 1024 * 1024;

/// What a thread must have left for one level of a text if the read is to stay on it.
/// Unoptimised: 2.3 times the costliest type of the crates (55 KiB, a struct that holds
/// itself in a box: `RawTypeDesc`), 1.5 times the costliest shape there is. Optimised: 2.9
/// times the costliest shape (11 KiB).
const TO_STAY_A_LEVEL: usize = if UNOPTIMISED { 128 * 1024 } else { 32 * 1024 };

/// What a thread must have left for a read whatever the text, if the read is to stay on it.
const TO_STAY_FOR_A_READ: usize = if UNOPTIMISED { 1024 * 1024 } else { 512 * 1024 };

/// The stack made for facet's walk of a text or value that nests `levels` deep.
pub(super) const fn made(levels: usize) -> usize {
    MADE_FOR_A_READ.saturating_add(levels.saturating_mul(MADE_A_LEVEL))
}

/// The stack a thread must have left for facet's walk of a text or value that nests
/// `levels` deep to run on it.
pub(super) const fn to_stay(levels: usize) -> usize {
    TO_STAY_FOR_A_READ.saturating_add(levels.saturating_mul(TO_STAY_A_LEVEL))
}

/// Runs `walk`, facet's read of a typed target from a text that nests `levels` deep (or its
/// walk of a value that deep), with the stack it may take: on the thread's own where
/// [`to_stay`] of it is left, else on one of [`made`] bytes.
#[cfg(not(target_family = "wasm"))]
pub(super) fn with_stack_for<R>(levels: usize, walk: impl FnOnce() -> R) -> R {
    stacker::maybe_grow(to_stay(levels), made(levels), walk)
}

/// Runs `walk` on the stack there is. A WebAssembly module has one stack, sized when the
/// module is linked, and no other can be made with a guard below it: the embedder sizes it
/// (`MAX_JSON_DEPTH`).
#[cfg(target_family = "wasm")]
pub(super) fn with_stack_for<R>(levels: usize, walk: impl FnOnce() -> R) -> R {
    let _ = (to_stay(levels), made(levels));
    walk()
}

/// Whether the thread has less stack left than one more level of a walk of facet's may take
/// with what a leaf runs: where a walk that can start again gives up (the writer's). A
/// thread whose stack is not known is one that has little.
#[cfg(not(target_family = "wasm"))]
pub(super) fn is_low() -> bool {
    stacker::remaining_stack().is_none_or(|left| left < to_stay(1))
}

/// Never: there is no other stack to go to.
#[cfg(target_family = "wasm")]
pub(super) fn is_low() -> bool {
    false
}

#[cfg(test)]
#[allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "the test runs each case in a child process, on a thread of a stack it chooses"
)]
mod tests {
    use std::process::{Command, Stdio};

    use facet::Facet;
    use indexmap::IndexMap;

    use super::to_stay;
    use crate::json::Value;

    /// A struct with an untyped member.
    #[derive(Facet)]
    struct Sidecar {
        #[facet(default)]
        a: Option<Value>,
    }

    /// A struct that holds itself in a list (`DeprecatedDefinitionEvent`'s shape).
    #[derive(Facet)]
    struct Tree {
        #[facet(default)]
        a: Option<Vec<Tree>>,
    }

    /// A struct that holds itself in a map.
    #[derive(Facet)]
    struct Named {
        #[facet(default)]
        a: Option<IndexMap<String, Option<Named>>>,
    }

    /// A struct that holds itself in a box (`RawTypeDesc`'s shape): the costliest a level of
    /// the types the crates have.
    #[derive(Facet)]
    struct Chain {
        #[facet(default)]
        a: Option<Box<Chain>>,
    }

    /// An untagged enum that holds itself in a list (`molang::list::ListForm`'s shape).
    #[derive(Facet)]
    #[facet(untagged)]
    #[repr(u8)]
    #[allow(dead_code, reason = "read, never looked at")]
    enum Lists {
        Many(Vec<Lists>),
        One(u8),
    }

    /// An untagged enum, a box and a struct for each level: no type of the crates, the
    /// costliest shape found.
    #[derive(Facet)]
    #[facet(untagged)]
    #[repr(u8)]
    #[allow(dead_code, reason = "read, never looked at")]
    enum Either {
        Many(Vec<Either>),
        One {
            #[facet(default)]
            a: Option<Box<Either>>,
        },
    }

    /// `open` `levels` times around `bottom`, closed.
    fn nest(levels: usize, open: fn(usize) -> &'static str, bottom: &str) -> String {
        let mut text = String::new();
        for level in 0..levels {
            text.push_str(open(level));
        }
        text.push_str(bottom);
        for level in (0..levels).rev() {
            text.push(if open(level).starts_with('[') {
                ']'
            } else {
                '}'
            });
        }
        text
    }

    fn objects(_: usize) -> &'static str {
        "{\"a\":"
    }

    fn mixed(level: usize) -> &'static str {
        if level.is_multiple_of(2) {
            "{\"a\":"
        } else {
            "["
        }
    }

    /// Whether facet reads `T` from `text`, called as the door calls it but on the stack
    /// there is.
    fn reads<T: for<'f> Facet<'f>>(text: &str) -> bool {
        facet_json::from_str_jsonc::<T>(text).is_ok()
    }

    /// A walk of facet's by name: the share of [`to_stay`] its thread is given, as a
    /// divisor and a multiplier, and whether the walk of a text or value `levels` deep ends
    /// well.
    type Walk = (&'static str, (usize, usize), fn(usize) -> bool);

    /// The walks the door lets run on a thread's own stack, each on half of what it asks
    /// the thread to have left: a margin of two in the build the test is built in. The
    /// last is no type of the crates: it has three quarters.
    const WALKS: &[Walk] = &[
        ("a struct with an untyped member", (1, 2), |levels| {
            reads::<Sidecar>(&nest(levels, objects, "7"))
        }),
        ("a struct that holds itself in a list", (1, 2), |levels| {
            let bottom = if levels.is_multiple_of(2) { "" } else { "null" };
            reads::<Tree>(&nest(levels, mixed, bottom))
        }),
        ("a struct that holds itself in a map", (1, 2), |levels| {
            reads::<Named>(&nest(levels, objects, "null"))
        }),
        ("a struct that holds itself in a box", (1, 2), |levels| {
            reads::<Chain>(&nest(levels, objects, "null"))
        }),
        (
            "an untagged enum that holds itself in a list",
            (1, 2),
            |levels| reads::<Lists>(&nest(levels, |_| "[", "7")),
        ),
        ("a value written by facet-json", (1, 2), |levels| {
            let Ok(value) = crate::json::from_str_jsonc::<Value>(&nest(levels, mixed, "7")) else {
                return false;
            };
            facet_json::to_string(&value).is_ok()
        }),
        (
            "an untagged enum, a box and a struct a level",
            (3, 4),
            |levels| reads::<Either>(&nest(levels, objects, "null")),
        ),
    ];

    /// The child of the test below: one walk at one depth, on a thread with its share of
    /// the stack the door asks a thread to have left. Without the variable it is no child.
    #[test]
    #[ignore = "no test of its own: the child process of the test below, which runs it with --ignored"]
    fn child() {
        let Ok(case) = std::env::var("JSON_STACK_MARGIN") else {
            return;
        };
        let mut parts = case.split(' ').map(|part| part.parse::<usize>().unwrap());
        let (walk, levels) = (parts.next().unwrap(), parts.next().unwrap());
        let (_, (times, of), run) = WALKS[walk];
        let ended = std::thread::Builder::new()
            .stack_size(to_stay(levels) * times / of)
            .spawn(move || run(levels))
            .unwrap()
            .join()
            .unwrap();
        assert!(ended, "JSON_STACK_MARGIN: the walk failed");
    }

    /// What a thread must have left for a walk to stay on it is twice what the walk takes,
    /// in the build at hand (run with `--release` for the optimised sizes): each walk ends
    /// well on half of it, at one level, at the depth of an ordinary file and at the limit.
    /// A stack that overflows ends the child, not this test.
    #[test]
    fn a_walk_that_stays_on_the_thread_s_stack_has_twice_what_it_takes() {
        let mut failures = Vec::new();
        for (walk, (name, _, _)) in WALKS.iter().enumerate() {
            for levels in [1, 16, crate::json::MAX_JSON_DEPTH] {
                let output = Command::new(std::env::current_exe().unwrap())
                    .args([
                        "--exact",
                        "json::stack::tests::child",
                        "--ignored",
                        "--test-threads=1",
                        "--nocapture",
                    ])
                    .env("JSON_STACK_MARGIN", format!("{walk} {levels}"))
                    .stdout(Stdio::null())
                    .stderr(Stdio::piped())
                    .output()
                    .unwrap();
                if !output.status.success() {
                    let said = String::from_utf8_lossy(&output.stderr);
                    let why = said
                        .lines()
                        .find(|line| line.contains("overflow") || line.contains("JSON_STACK"))
                        .unwrap_or("no message");
                    failures.push(format!(
                        "{name}, {levels} levels: {} ({why})",
                        output.status
                    ));
                }
            }
        }
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }

    /// The stack made is never the smaller of the two.
    #[test]
    fn the_stack_made_is_larger_than_the_stack_to_stay_on() {
        for levels in [0, 1, 16, 128, 100_000] {
            assert!(super::made(levels) >= to_stay(levels));
        }
    }
}
