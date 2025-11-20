use std::{
    cmp::Ordering,
    ops::{Bound, RangeBounds},
};

use crate::{
    rand::{
        distributions::{
            uniform::{SampleBorrow, SampleUniform, Uniform},
            Distribution,
        },
        thread_rng, Rng,
    },
    Bounded,
};

/// Generate a random value in the range [`min`, `max_exclusive`) with a new lazily-initialized thread-local random number generator.
///
/// Panics if `min >= max_exclusive`.
#[inline]
pub fn random_exclusively<X: SampleUniform, B1: SampleBorrow<X>, B2: SampleBorrow<X>>(
    min: B1,
    max_exclusive: B2,
) -> X {
    random_exclusively_with_rng(min, max_exclusive, &mut thread_rng())
}

/// Generate a random value in the range [`min`, `max_exclusive`) with an existing random number generator.
///
/// Panics if `min >= max_exclusive`.
#[inline]
pub fn random_exclusively_with_rng<
    X: SampleUniform,
    B1: SampleBorrow<X>,
    B2: SampleBorrow<X>,
    T: Rng,
>(
    min: B1,
    max_exclusive: B2,
    rng: &mut T,
) -> X {
    let uniform = Uniform::new(min, max_exclusive);

    uniform.sample(rng)
}

/// Generate a random value in the range [`min`, `max_inclusive`] with a new lazily-initialized thread-local random number generator.
///
/// Panics if `min > max_inclusive`.
#[inline]
pub fn random_inclusively<X: SampleUniform, B1: SampleBorrow<X>, B2: SampleBorrow<X>>(
    min: B1,
    max_inclusive: B2,
) -> X {
    random_inclusively_with_rng(min, max_inclusive, &mut thread_rng())
}

/// Generate a random value in the range [`min`, `max_inclusive`] with an existing random number generator.
///
/// Panics if `min > max_inclusive`.
#[inline]
pub fn random_inclusively_with_rng<
    X: SampleUniform,
    B1: SampleBorrow<X>,
    B2: SampleBorrow<X>,
    T: Rng,
>(
    min: B1,
    max_inclusive: B2,
    rng: &mut T,
) -> X {
    Uniform::new_inclusive(min, max_inclusive).sample(rng)
}

/// Generate a random value in the range [`a`, `b`] or [`b`, `a`] with a new lazily-initialized thread-local random number generator.
///
/// Panics if a and b can not be compared.
#[inline]
pub fn random_inclusively_cmp<X: SampleUniform + Ord + Clone, B: SampleBorrow<X>>(a: B, b: B) -> X {
    random_inclusively_cmp_with_rng(a, b, &mut thread_rng())
}

/// Generate a random value in the range [`a`, `b`] or [`b`, `a`] with an existing random number generator.
#[inline]
pub fn random_inclusively_cmp_with_rng<
    X: SampleUniform + Ord + Clone,
    B: SampleBorrow<X>,
    T: Rng,
>(
    a: B,
    b: B,
    rng: &mut T,
) -> X {
    match a.borrow().cmp(b.borrow()) {
        Ordering::Greater => random_inclusively_with_rng(b, a, rng),
        Ordering::Equal => a.borrow().clone(),
        Ordering::Less => random_inclusively_with_rng(a, b, rng),
    }
}

/// Generate a random value in the range of the output type with a new lazily-initialized thread-local random number generator.
#[inline]
pub fn random<X: SampleUniform + Bounded>() -> X {
    random_with_rng(&mut thread_rng())
}

/// Generate a random value in the range of the output type with an existing random number generator.
#[inline]
pub fn random_with_rng<X: SampleUniform + Bounded, T: Rng>(rng: &mut T) -> X {
    random_inclusively_with_rng(X::min_value(), X::max_value(), rng)
}

/// Generate a random value in the range [`min`, `Bounded::max_value()`] with a new lazily-initialized thread-local random number generator.
#[inline]
pub fn random_at_least<X: SampleUniform + Bounded, B: SampleBorrow<X>>(min: B) -> X {
    random_at_least_with_rng(min, &mut thread_rng())
}

/// Generate a random value in the range [`min`, `X::max_value()`] with an existing random number generator.
#[inline]
pub fn random_at_least_with_rng<X: SampleUniform + Bounded, B: SampleBorrow<X>, T: Rng>(
    min: B,
    rng: &mut T,
) -> X {
    random_inclusively_with_rng(min, X::max_value(), rng)
}

/// Generate a random value in the range [`X::min_value()`, `max_inclusive`] with a new lazily-initialized thread-local random number generator.
#[inline]
pub fn random_at_most<X: SampleUniform + Bounded, B: SampleBorrow<X>>(max_inclusive: B) -> X {
    random_at_most_with_rng(max_inclusive, &mut thread_rng())
}

/// Generate a random value in the range [`X::min_value()`, `max_inclusive`] with an existing random number generator.
#[inline]
pub fn random_at_most_with_rng<X: SampleUniform + Bounded, B: SampleBorrow<X>, T: Rng>(
    max_inclusive: B,
    rng: &mut T,
) -> X {
    random_inclusively_with_rng(X::min_value(), max_inclusive, rng)
}

/// Generate a random value in the range [`X::min_value()`, `max_exclusive`) with a new lazily-initialized thread-local random number generator.
///
/// Panics if X::min_value() == max_exclusive.
#[inline]
pub fn random_at_most_exclusively<X: SampleUniform + Bounded, B: SampleBorrow<X>>(
    max_exclusive: B,
) -> X {
    random_at_most_exclusively_with_rng(max_exclusive, &mut thread_rng())
}

/// Generate a random value in the range [`X::min_value()`, `max_exclusive`) with an existing random number generator.
///
/// Panics if X::min_value() == max_exclusive.
#[inline]
pub fn random_at_most_exclusively_with_rng<
    X: SampleUniform + Bounded,
    B: SampleBorrow<X>,
    T: Rng,
>(
    max_exclusive: B,
    rng: &mut T,
) -> X {
    random_exclusively_with_rng(X::min_value(), max_exclusive, rng)
}

/// Generate a random value in a specific range with a new lazily-initialized thread-local random number generator.
///
/// Panics if the start bound is exclusive.
#[inline]
pub fn random_ranged<X: SampleUniform + Bounded, R: RangeBounds<X>>(range: R) -> X {
    random_ranged_with_rng(range, &mut thread_rng())
}

/// Generate a random value in a specific range with an existing random number generator.
///
/// Panics if the start bound is exclusive.
#[inline]
pub fn random_ranged_with_rng<X: SampleUniform + Bounded, R: RangeBounds<X>, T: Rng>(
    range: R,
    rng: &mut T,
) -> X {
    let start = range.start_bound();
    let end = range.end_bound();

    match start {
        Bound::Excluded(_) => {
            panic!("random_ranged_with_rng called with a start bound which is exclusive")
        },
        Bound::Included(min) => match end {
            Bound::Excluded(max_exclusive) => random_exclusively_with_rng(min, max_exclusive, rng),
            Bound::Included(max_inclusive) => random_inclusively_with_rng(min, max_inclusive, rng),
            Bound::Unbounded => random_at_least_with_rng(min, rng),
        },
        Bound::Unbounded => match end {
            Bound::Excluded(max_exclusive) => {
                random_at_most_exclusively_with_rng(max_exclusive, rng)
            },
            Bound::Included(max_inclusive) => random_at_most_with_rng(max_inclusive, rng),
            Bound::Unbounded => random_with_rng(rng),
        },
    }
}
