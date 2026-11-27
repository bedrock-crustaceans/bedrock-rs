//! The one protocol of every cost test of this crate: a function is run on an input of
//! `n` entries and on one of four times as many, and the larger run must take under eight
//! times the smaller one.
//!
//! Four times the entries are four times the time when the function is linear (five with a
//! logarithm or a colder cache) and sixteen times when it is quadratic, so that a neighbour
//! that slows one of the two measurements by half does not cross the bound either way. No
//! absolute time is asked for.
//!
//! - The inputs are built before the clock starts; only the call of the function is timed, and
//!   what it returns is dropped after the clock has stopped.
//! - Each size is the best of three runs, the two sizes taken in turn so that both see the
//!   same neighbours.
//! - A measurement over the bound is taken once more before it is believed.
//! - One lock for the whole process is held from the building of the inputs to the last
//!   measurement: the cost tests of one test binary never measure, or build, beside each other.
//! - Every measurement prints its ratio (`cost: <what>: … ratio <r>`), the first and any second.
//!
//! Choose `n` so that the smaller run takes about 20 ms or more in the dev profile and the
//! whole test stays in seconds. A test whose larger input would take minutes lowers `n`, never
//! the factor.
//!
//! There is one copy of this file, here, compiled for the unit tests of this crate and behind
//! the `test-cost` feature for everything else. The in-crate tests reach it as
//! `crate::test_cost`; an integration-test binary, here or in another crate, names
//! `bedrock_addon_core::test_cost` and turns the feature on in its `[dev-dependencies]`
//! (`bedrock_addon_core = { path = "...", features = ["test-cost"] }`). A test binary links the
//! library once, so it has the one lock.

#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "a cost test reads the clock; library code never does"
)]
#![allow(dead_code, reason = "each test binary uses part of the helper")]

use std::hint::black_box;
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

/// How many times the entries of the smaller input the larger one has.
pub const TIMES: usize = 4;

/// The larger input must take under this many times the smaller one's time: linear is
/// [`TIMES`], quadratic its square.
pub const BOUND: u32 = 8;

/// One cost test of the process at a time.
static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

/// The lock of the process's cost tests. A test that failed while it held the lock does not
/// fail the ones after it.
fn alone() -> MutexGuard<'static, ()> {
    ONE_AT_A_TIME.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Runs `measure` under the lock of the cost tests: for the rare test that compares two times
/// that are not two sizes of one input (it reads the clock with [`timed`] or [`time_of`]).
pub fn alone_for<T>(measure: impl FnOnce() -> T) -> T {
    let _alone = alone();
    measure()
}

/// Requires `step` on `input(4 n)` to take under eight times `step` on `input(n)`. Both
/// inputs are built once, before anything is measured, and `step` borrows them.
#[track_caller]
pub fn assert_linear_at<I, T>(
    what: &str,
    n: usize,
    mut input: impl FnMut(usize) -> I,
    mut step: impl FnMut(&I) -> T,
) {
    let _alone = alone();
    let inputs = [input(n), input(TIMES * n)];
    measured([what], n, |slot| [time_of(|| step(&inputs[slot]))]);
}

/// One function of [`assert_each_linear_at`]: what it is called in the measurement's line, and
/// the function, which checks what it did itself.
pub type Step<'a, I> = (&'a str, &'a dyn Fn(&I));

/// [`assert_linear_at`] for several functions on the same two inputs, which are built once:
/// each `(what, step)` is held to the bound on its own, one after the other, and the lock is
/// held from the building of the inputs to the last of them.
#[track_caller]
pub fn assert_each_linear_at<I>(
    n: usize,
    mut input: impl FnMut(usize) -> I,
    steps: &[Step<'_, I>],
) {
    let _alone = alone();
    let inputs = [input(n), input(TIMES * n)];
    for (what, step) in steps {
        measured([*what], n, |slot| [time_of(|| step(&inputs[slot]))]);
    }
}

/// [`assert_linear_at`] for a `step` that consumes its input: `input` builds a fresh one
/// before each run, outside the measured time.
#[track_caller]
pub fn assert_linear_consuming_at<I, T>(
    what: &str,
    n: usize,
    mut input: impl FnMut(usize) -> I,
    mut step: impl FnMut(I) -> T,
) {
    let _alone = alone();
    let sizes = [n, TIMES * n];
    measured([what], n, |slot| {
        let input = input(sizes[slot]);
        [time_of(|| step(input))]
    });
}

/// [`assert_linear_at`] for a test that looks at what the function returned: `run` is handed
/// a size, builds the input of that size, takes the time of the one call with [`timed`],
/// checks the result and returns the time. It is called three times for each size.
#[track_caller]
pub fn assert_linear_timed_at(what: &str, n: usize, mut run: impl FnMut(usize) -> Duration) {
    assert_each_linear_timed_at([what], n, |size| [run(size)]);
}

/// [`assert_linear_timed_at`] for several functions that share one input: `run` builds the
/// input of the size once and returns the time of one call of each function, in the order of
/// `what`. Each function is held to the bound on its own.
#[track_caller]
pub fn assert_each_linear_timed_at<const N: usize>(
    what: [&str; N],
    n: usize,
    mut run: impl FnMut(usize) -> [Duration; N],
) {
    let _alone = alone();
    let sizes = [n, TIMES * n];
    measured(what, n, |slot| run(sizes[slot]));
}

/// What `step` returns and how long the call took. The clock stops before the result is
/// handed back, so that dropping it is not in the time.
pub fn timed<T>(step: impl FnOnce() -> T) -> (T, Duration) {
    let started = Instant::now();
    let output = black_box(step());
    let took = started.elapsed();
    (output, took)
}

/// How long one call of `step` takes; what it returns is dropped after the clock has stopped.
pub fn time_of<T>(step: impl FnOnce() -> T) -> Duration {
    timed(step).1
}

/// The measurement, for a caller that holds the lock: `run(0)` is the time of one call of
/// each function on the smaller input and `run(1)` on the larger.
#[track_caller]
fn measured<const N: usize>(
    what: [&str; N],
    n: usize,
    mut run: impl FnMut(usize) -> [Duration; N],
) {
    let mut failed = Vec::new();
    for _attempt in 0..2 {
        let mut best = [[Duration::MAX; N]; 2];
        for _ in 0..3 {
            for (slot, best) in best.iter_mut().enumerate() {
                for (best, took) in best.iter_mut().zip(run(slot)) {
                    *best = (*best).min(took);
                }
            }
        }
        let [single, larger] = best;
        let mut linear = true;
        for ((what, single), larger) in what.iter().zip(single).zip(larger) {
            let ratio = larger.as_secs_f64() / single.as_secs_f64().max(1e-9);
            eprintln!(
                "cost: {what}: {n} in {single:?}, {} in {larger:?}, ratio {ratio:.2}",
                TIMES * n
            );
            if larger >= single * BOUND {
                linear = false;
                failed.push((*what, single, larger));
            }
        }
        if linear {
            return;
        }
    }
    panic!(
        "{} entries took {BOUND} times {n} entries or more, twice: {failed:?}",
        TIMES * n
    );
}
