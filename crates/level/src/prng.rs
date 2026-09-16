//! MT19937, the 32-bit Mersenne Twister -- see the Phase 6 "Bedrock PRNG
//! helpers" task in `TODO.md`.
//!
//! # Scope, and what this module deliberately does not implement
//!
//! The task this module answers asks for MT19937 plus "the game's seed
//! hashing for slime-chunk / spawn reproducibility." Only the first half is
//! implemented here. Neither reference project this crate otherwise
//! consults for format details (see `TODO.md`'s References section)
//! implements, or even names, slime-chunk determination, spawn-position
//! hashing, or MT19937 at all -- both are format libraries (a world editor
//! and a converter), not world generators, and a full-text search of both
//! checkouts for slime/PRNG-related terms finds nothing. There is
//! consequently no reference-established formula to reimplement, and this
//! crate's own fixture corpus has no way to verify one either: no fixture
//! records anywhere a chunk was actually generated as a slime chunk (that
//! is a runtime classification the game computes on the fly from the seed,
//! not a stored flag), so there is no ground truth in this corpus a
//! candidate hash could be checked against. Per the task's own stated
//! fallback -- "if nothing in the corpus can verify it, pin only the
//! MT19937 reference vectors and report the slime hash as unverified" --
//! that hash is not implemented here rather than guessed. A further open
//! question worth recording for whoever picks this back up: it is not
//! actually established that Bedrock's slime-chunk determination *uses*
//! MT19937 at all, as opposed to the linear-congruential generator behind
//! `java.util.Random` that Java Edition's own slime-chunk check uses and
//! that Bedrock's world generator is known to preserve compatibility with
//! for other seeded systems. Implementing a slime-chunk hash correctly
//! needs that question answered first, from a source this crate can point
//! to, not assumed.
//!
//! # What is implemented
//!
//! [`Mt19937`] is the standard 32-bit Mersenne Twister -- the generator
//! `std::mt19937` names in C++, the same algorithm Makoto Matsumoto and
//! Takuji Nishimura published and placed in the public domain (their
//! reference implementation, `mt19937ar.c`, is itself public domain,
//! distinct from and not one of this crate's two format references).
//! [`Mt19937::new`] seeds from one `u32` (the reference implementation's
//! `init_genrand`); [`Mt19937::from_seed_array`] seeds from an arbitrary
//! slice of `u32` words (`init_by_array`), the routine the reference
//! implementation itself recommends for real use because a single 32-bit
//! seed does not exercise the state array's full mixing. Both are pinned
//! in this module's tests against reference vectors: `init_genrand(5489)`
//! against the published reference vector from `mt19937ar.c`'s own
//! `mt19937ar.out` sample output (5489 is that reference implementation's
//! documented default seed), and `init_by_array([5489])` against CPython's
//! `random` module, whose C implementation is a direct, independently
//! maintained port of the same reference algorithm and seeds an integer
//! the same way -- an independent cross-check that both seeding routines
//! here match a second, unrelated implementation bit for bit, not just
//! each other.
//!
//! This is a general-purpose PRNG primitive, not a Bedrock format record:
//! it reads and writes nothing on disk, and nothing else in this crate
//! calls it. It exists for a caller who wants a Bedrock-adjacent-quality
//! MT19937 to build their own seed-reproducibility logic on top of, per
//! the task -- once the slime-chunk (or spawn-position) hash itself is
//! actually established from a citable source.

/// Degree of recurrence: the size of the generator's state array.
const N: usize = 624;
/// Middle word, used in the twist recurrence.
const M: usize = 397;
/// Twist recurrence's XOR mask, applied when the dropped low bit is 1.
const MATRIX_A: u32 = 0x9908_b0df;
/// Most significant bit of a 32-bit word -- the "upper mask" in the
/// reference implementation's terminology.
const UPPER_MASK: u32 = 0x8000_0000;
/// The other 31 bits -- the "lower mask."
const LOWER_MASK: u32 = 0x7fff_ffff;

/// The standard 32-bit Mersenne Twister (`std::mt19937`/`mt19937ar`). See
/// the module docs for what this is pinned against and what it is not
/// (a slime-chunk or spawn-position hash).
#[derive(Debug, Clone)]
pub struct Mt19937 {
    state: [u32; N],
    /// Index of the next word to return from `state`; `N + 1` is the
    /// reference implementation's own sentinel for "never seeded," which
    /// can't occur here since both constructors seed immediately, but the
    /// field keeps the same shape as the reference algorithm rather than
    /// asserting a narrower invariant that buys nothing.
    index: usize,
}

impl Mt19937 {
    /// Seeds from a single 32-bit value -- the reference implementation's
    /// `init_genrand`. [`Self::from_seed_array`] is the reference
    /// implementation's own recommended choice for real use (see the
    /// module docs); this constructor exists because it is what a bare
    /// `u32` seed needs, and because it is what the `init_genrand(5489)`
    /// reference vector this module pins against was generated with.
    #[must_use]
    pub fn new(seed: u32) -> Self {
        let mut state = [0u32; N];
        state[0] = seed;
        for i in 1..N {
            state[i] = 1_812_433_253u32
                .wrapping_mul(state[i - 1] ^ (state[i - 1] >> 30))
                .wrapping_add(i as u32);
        }
        Self { state, index: N }
    }

    /// Seeds from an arbitrary-length key of 32-bit words -- the reference
    /// implementation's `init_by_array`. An empty `key` behaves as the
    /// reference implementation itself does: the mixing loop below simply
    /// never advances `j` past 0, reusing `key[0]` -- which would panic on
    /// an empty slice, so this rejects that case up front instead of
    /// replicating undefined behaviour from a C implementation that never
    /// has to answer the question (its caller always passes a real key
    /// array).
    ///
    /// # Panics
    ///
    /// Panics if `key` is empty.
    #[must_use]
    pub fn from_seed_array(key: &[u32]) -> Self {
        assert!(!key.is_empty(), "MT19937 seed array must not be empty");

        let mut rng = Self::new(19_650_218);
        let mut i = 1usize;
        let mut j = 0usize;
        for _ in 0..N.max(key.len()) {
            rng.state[i] = (rng.state[i]
                ^ (rng.state[i - 1] ^ (rng.state[i - 1] >> 30)).wrapping_mul(1_664_525))
            .wrapping_add(key[j])
            .wrapping_add(j as u32);
            i += 1;
            j += 1;
            if i >= N {
                rng.state[0] = rng.state[N - 1];
                i = 1;
            }
            if j >= key.len() {
                j = 0;
            }
        }
        for _ in 0..N - 1 {
            rng.state[i] = (rng.state[i]
                ^ (rng.state[i - 1] ^ (rng.state[i - 1] >> 30)).wrapping_mul(1_566_083_941))
            .wrapping_sub(i as u32);
            i += 1;
            if i >= N {
                rng.state[0] = rng.state[N - 1];
                i = 1;
            }
        }
        rng.state[0] = 0x8000_0000;
        rng
    }

    /// Regenerates all `N` words of state via the twist recurrence --
    /// called automatically by [`Self::next_u32`] every `N` draws.
    fn twist(&mut self) {
        for kk in 0..N {
            let y = (self.state[kk] & UPPER_MASK) | (self.state[(kk + 1) % N] & LOWER_MASK);
            let mut next = self.state[(kk + M) % N] ^ (y >> 1);
            if y & 1 != 0 {
                next ^= MATRIX_A;
            }
            self.state[kk] = next;
        }
        self.index = 0;
    }

    /// Draws the next 32-bit output, tempering the raw state word exactly
    /// as the reference implementation does.
    pub fn next_u32(&mut self) -> u32 {
        if self.index >= N {
            self.twist();
        }
        let mut y = self.state[self.index];
        self.index += 1;

        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c_5680;
        y ^= (y << 15) & 0xefc6_0000;
        y ^= y >> 18;
        y
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `init_genrand(5489)`'s first ten outputs, from the reference
    /// implementation's own published `mt19937ar.out` (5489 is
    /// `mt19937ar.c`'s documented default seed) -- independently
    /// re-derived and cross-checked against a from-spec Python port during
    /// review, not transcribed from memory alone.
    #[test]
    fn init_genrand_matches_the_published_reference_vector() {
        let mut rng = Mt19937::new(5489);
        let expected = [
            3_499_211_612u32,
            581_869_302,
            3_890_346_734,
            3_586_334_585,
            545_404_204,
            4_161_255_391,
            3_922_919_429,
            949_333_985,
            2_715_962_298,
            1_323_567_403,
        ];
        for want in expected {
            assert_eq!(rng.next_u32(), want);
        }
    }

    /// `init_by_array([5489])`'s first outputs, cross-checked against
    /// CPython's `random` module (`random.seed(5489)`;
    /// `random.getrandbits(32)`) -- an independent implementation of the
    /// same reference algorithm, not derived from this module's own code.
    #[test]
    fn init_by_array_matches_an_independent_implementation() {
        let mut rng = Mt19937::from_seed_array(&[5489]);
        let expected = [
            3_382_763_572u32,
            956_215_839,
            417_760_592,
            166_104_981,
            4_181_578_304,
        ];
        for want in expected {
            assert_eq!(rng.next_u32(), want);
        }
    }

    /// Same cross-check, seed 0 -- covers the all-zero key path through
    /// `from_seed_array`'s mixing loop.
    #[test]
    fn init_by_array_matches_an_independent_implementation_seed_zero() {
        let mut rng = Mt19937::from_seed_array(&[0]);
        let expected = [
            3_626_764_237u32,
            1_654_615_998,
            3_255_389_356,
            3_823_568_514,
            1_806_341_205,
        ];
        for want in expected {
            assert_eq!(rng.next_u32(), want);
        }
    }

    /// Same cross-check, seed 1.
    #[test]
    fn init_by_array_matches_an_independent_implementation_seed_one() {
        let mut rng = Mt19937::from_seed_array(&[1]);
        let expected = [
            577_090_037u32,
            2_444_712_010,
            3_639_700_191,
            3_445_702_192,
            3_280_387_012,
        ];
        for want in expected {
            assert_eq!(rng.next_u32(), want);
        }
    }

    #[test]
    fn same_seed_produces_the_same_sequence() {
        let mut a = Mt19937::new(42);
        let mut b = Mt19937::new(42);
        for _ in 0..2000 {
            assert_eq!(a.next_u32(), b.next_u32());
        }
    }

    #[test]
    fn different_seeds_diverge() {
        let mut a = Mt19937::new(1);
        let mut b = Mt19937::new(2);
        assert_ne!(a.next_u32(), b.next_u32());
    }

    #[test]
    #[should_panic(expected = "must not be empty")]
    fn from_seed_array_rejects_an_empty_key() {
        let _ = Mt19937::from_seed_array(&[]);
    }

    /// A run long enough to force several twist regenerations (every 624
    /// draws), confirming the recurrence keeps matching the reference
    /// sequence past the first block of state.
    #[test]
    fn stays_consistent_across_multiple_twists() {
        let mut rng = Mt19937::new(5489);
        for _ in 0..624 {
            rng.next_u32();
        }
        // 625th output (index 624, first word of the second twist) from
        // the same independently re-derived reference vector used above.
        assert_eq!(rng.next_u32(), 4_178_893_912);
    }
}
