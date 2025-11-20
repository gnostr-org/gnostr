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

/// Generate random values in the range [`min`, `max_exclusive`) with a new lazily-initialized thread-local random number generator.
///
/// Panics if `min >= max_exclusive`.
#[inline]
pub fn random_fill_exclusively<X: SampleUniform, B1: SampleBorrow<X>, B2: SampleBorrow<X>>(
    out: &mut [X],
    min: B1,
    max_exclusive: B2,
) {
    random_fill_exclusively_with_rng(out, min, max_exclusive, &mut thread_rng())
}

/// Generate random values in the range [`min`, `max_exclusive`) with an existing random number generator.
///
/// Panics if `min >= max_exclusive`.
#[inline]
pub fn random_fill_exclusively_with_rng<
    X: SampleUniform,
    B1: SampleBorrow<X>,
    B2: SampleBorrow<X>,
    T: Rng,
>(
    out: &mut [X],
    min: B1,
    max_exclusive: B2,
    rng: &mut T,
) {
    let uniform = Uniform::new(min, max_exclusive);

    for x in out.iter_mut() {
        *x = uniform.sample(rng);
    }
}

/// Generate random values in the range [`min`, `max_inclusive`] with a new lazily-initialized thread-local random number generator.
///
/// Panics if `min > max_inclusive`.
#[inline]
pub fn random_fill_inclusively<X: SampleUniform, B1: SampleBorrow<X>, B2: SampleBorrow<X>>(
    out: &mut [X],
    min: B1,
    max_inclusive: B2,
) {
    random_fill_inclusively_with_rng(out, min, max_inclusive, &mut thread_rng());
}

/// Generate random values in the range [`min`, `max_inclusive`] with an existing random number generator.
///
/// Panics if `min > max_inclusive`.
#[inline]
pub fn random_fill_inclusively_with_rng<
    X: SampleUniform,
    B1: SampleBorrow<X>,
    B2: SampleBorrow<X>,
    T: Rng,
>(
    out: &mut [X],
    min: B1,
    max_inclusive: B2,
    rng: &mut T,
) {
    let uniform = Uniform::new_inclusive(min, max_inclusive);
    let mut random_iter = uniform.sample_iter(rng);

    for x in out.iter_mut() {
        *x = random_iter.next().unwrap();
    }
}

/// Generate random values in the range [`a`, `b`] or [`b`, `a`] with a new lazily-initialized thread-local random number generator.
///
/// Panics if a and b can not be compared.
#[inline]
pub fn random_fill_inclusively_cmp<X: SampleUniform + Ord + Clone, B: SampleBorrow<X>>(
    out: &mut [X],
    a: B,
    b: B,
) {
    random_fill_inclusively_cmp_with_rng(out, a, b, &mut thread_rng());
}

/// Generate random values in the range [`a`, `b`] or [`b`, `a`] with an existing random number generator.
#[inline]
pub fn random_fill_inclusively_cmp_with_rng<
    X: SampleUniform + Ord + Clone,
    B: SampleBorrow<X>,
    T: Rng,
>(
    out: &mut [X],
    a: B,
    b: B,
    rng: &mut T,
) {
    match a.borrow().cmp(b.borrow()) {
        Ordering::Greater => random_fill_inclusively_with_rng(out, b, a, rng),
        Ordering::Equal => {
            for x in out.iter_mut() {
                *x = a.borrow().clone();
            }
        },
        Ordering::Less => random_fill_inclusively_with_rng(out, a, b, rng),
    }
}

/// Generate random values in the range of the output type with a new lazily-initialized thread-local random number generator.
#[inline]
pub fn random_fill<X: SampleUniform + Bounded>(out: &mut [X]) {
    random_fill_with_rng(out, &mut thread_rng())
}

/// Generate random values in the range of the output type with an existing random number generator.
#[inline]
pub fn random_fill_with_rng<X: SampleUniform + Bounded, T: Rng>(out: &mut [X], rng: &mut T) {
    random_fill_inclusively_with_rng(out, X::min_value(), X::max_value(), rng);
}

/// Generate random values in the range [`min`, `Bounded::max_value()`] with a new lazily-initialized thread-local random number generator.
#[inline]
pub fn random_fill_at_least<X: SampleUniform + Bounded, B: SampleBorrow<X>>(out: &mut [X], min: B) {
    random_fill_at_least_with_rng(out, min, &mut thread_rng());
}

/// Generate random values in the range [`min`, `X::max_value()`] with an existing random number generator.
#[inline]
pub fn random_fill_at_least_with_rng<X: SampleUniform + Bounded, B: SampleBorrow<X>, T: Rng>(
    out: &mut [X],
    min: B,
    rng: &mut T,
) {
    random_fill_inclusively_with_rng(out, min, X::max_value(), rng);
}

/// Generate random values in the range [`X::min_value()`, `max_inclusive`] with a new lazily-initialized thread-local random number generator.
#[inline]
pub fn random_fill_at_most<X: SampleUniform + Bounded, B: SampleBorrow<X>>(
    out: &mut [X],
    max_inclusive: B,
) {
    random_fill_at_most_with_rng(out, max_inclusive, &mut thread_rng());
}

/// Generate random values in the range [`X::min_value()`, `max_inclusive`] with an existing random number generator.
#[inline]
pub fn random_fill_at_most_with_rng<X: SampleUniform + Bounded, B: SampleBorrow<X>, T: Rng>(
    out: &mut [X],
    max_inclusive: B,
    rng: &mut T,
) {
    random_fill_inclusively_with_rng(out, X::min_value(), max_inclusive, rng);
}

/// Generate random values in the range [`X::min_value()`, `max_exclusive`) with a new lazily-initialized thread-local random number generator.
///
/// Panics if X::min_value() == max_exclusive.
#[inline]
pub fn random_fill_at_most_exclusively<X: SampleUniform + Bounded, B: SampleBorrow<X>>(
    out: &mut [X],
    max_exclusive: B,
) {
    random_fill_at_most_exclusively_with_rng(out, max_exclusive, &mut thread_rng());
}

/// Generate random values in the range [`X::min_value()`, `max_exclusive`) with an existing random number generator.
///
/// Panics if X::min_value() == max_exclusive.
#[inline]
pub fn random_fill_at_most_exclusively_with_rng<
    X: SampleUniform + Bounded,
    B: SampleBorrow<X>,
    T: Rng,
>(
    out: &mut [X],
    max_exclusive: B,
    rng: &mut T,
) {
    random_fill_exclusively_with_rng(out, X::min_value(), max_exclusive, rng);
}

/// Generate random values in a specific range with a new lazily-initialized thread-local random number generator.
///
/// Panics if the start bound is exclusive.
#[inline]
pub fn random_fill_ranged<X: SampleUniform + Bounded, R: RangeBounds<X>>(out: &mut [X], range: R) {
    random_fill_ranged_with_rng(out, range, &mut thread_rng())
}

/// Generate random values in a specific range with an existing random number generator.
///
/// Panics if the start bound is exclusive.
#[inline]
pub fn random_fill_ranged_with_rng<X: SampleUniform + Bounded, R: RangeBounds<X>, T: Rng>(
    out: &mut [X],
    range: R,
    rng: &mut T,
) {
    let start = range.start_bound();
    let end = range.end_bound();

    match start {
        Bound::Excluded(_) => {
            panic!("random_fill_ranged_with_rng called with a start bound which is exclusive")
        },
        Bound::Included(min) => match end {
            Bound::Excluded(max_exclusive) => {
                random_fill_exclusively_with_rng(out, min, max_exclusive, rng)
            },
            Bound::Included(max_inclusive) => {
                random_fill_inclusively_with_rng(out, min, max_inclusive, rng)
            },
            Bound::Unbounded => random_fill_at_least_with_rng(out, min, rng),
        },
        Bound::Unbounded => match end {
            Bound::Excluded(max_exclusive) => {
                random_fill_at_most_exclusively_with_rng(out, max_exclusive, rng)
            },
            Bound::Included(max_inclusive) => random_fill_at_most_with_rng(out, max_inclusive, rng),
            Bound::Unbounded => random_fill_with_rng(out, rng),
        },
    }
}
