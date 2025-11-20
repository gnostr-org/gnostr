/// The random range of different types.
pub trait Bounded {
    fn max_value() -> Self;
    fn min_value() -> Self;
}

macro_rules! bounded_impl {
    ($t:ident) => {
        impl Bounded for $t {
            #[inline]
            fn max_value() -> Self {
                $t::MAX
            }

            #[inline]
            fn min_value() -> Self {
                $t::MIN
            }
        }
    };
}

bounded_impl!(u8);
bounded_impl!(u16);
bounded_impl!(u32);
bounded_impl!(u64);
bounded_impl!(u128);
bounded_impl!(usize);
bounded_impl!(i8);
bounded_impl!(i16);
bounded_impl!(i32);
bounded_impl!(i64);
bounded_impl!(i128);
bounded_impl!(isize);

impl Bounded for f64 {
    #[inline]
    fn max_value() -> Self {
        1.0
    }

    #[inline]
    fn min_value() -> Self {
        0.0
    }
}

impl Bounded for f32 {
    #[inline]
    fn max_value() -> Self {
        1.0
    }

    #[inline]
    fn min_value() -> Self {
        0.0
    }
}
