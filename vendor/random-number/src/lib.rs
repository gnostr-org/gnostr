/*!
# Random Number

Generate random numbers quickly.

### The `random!` Marco

```rust
use random_number::random;

let n: u8 = random!();
println!("{}", n); // 0 ~ 255

let n: f64 = random!();
println!("{}", n); // 0.0 ~ 1.0

let n: u8 = random!(..=10);
println!("{}", n); // 0 ~ 10

let n: u8 = random!(..=9);
println!("{}", n); // 0 ~ 9

let n: u8 = random!(10..);
println!("{}", n); // 10 ~ 255

let n: i8 = random!(-2..=12);
println!("{}", n); // -2 ~ 12

let n: u8 = random!(12, 20);
println!("{}", n); // 12 ~ 20

let n: u8 = random!(20, 12);
println!("{}", n); // 12 ~ 20
```

The random number generator can be reused by adding it to the `random!` macro as the last argument.

```rust
use random_number::random;

let mut rng = random_number::rand::thread_rng();

let n: u8 = random!(rng);
println!("{}", n); // 0 ~ 255

let n: u8 = random!(..=10, rng);
println!("{}", n); // 0 ~ 10

let n: u8 = random!(20, 12, rng);
println!("{}", n); // 12 ~ 20
```

### The `random_ranged` Function

If the **range** is not literal, for example, a variable, `var_range`, storing an instance that implements the `RangeBounds` trait, the `var_range` variable cannot be used in the `random!` macro.

```rust,ignore
let var_range = 1..=10;

let n: u8 = random_number::random!(var_range); // compile error
```

In this case, use the `random_ranged` function instead.

```rust
let var_range = 1..=10;

let n: u8 = random_number::random_ranged(var_range);
println!("{}", n); // 1 ~ 10
```

### The `random_fill!` Marco

The `random_fill!` marco can be used to fill a slice with random numbers. The usage is like the `random!` macro. Just add a slice as the first argument when using the `random_fill!` macro.

```rust
let mut a = [0i8; 32];
random_number::random_fill!(a, -2..=12);

println!("{:?}", a);
```

### The `random_fill_ranged` Function

```rust
let var_range = 1..=10;

let mut a = [0u8; 32];
random_number::random_fill_ranged(&mut a, var_range);

println!("{:?}", a);
```
*/
pub extern crate rand;

mod bounded;
mod random_fill_functions;
mod random_functions;

pub use bounded::Bounded;
use proc_macro_hack::proc_macro_hack;
pub use random_fill_functions::*;
pub use random_functions::*;
/**
Generate a random number.

## Examples

```rust
# #![allow(unused_comparisons)]
let f: f64 = random_number::random!();

assert!(0.0 <= f && f <= 1.0);

let i: u8 = random_number::random!(..);

assert!(0 <= i && i <= 255);
```

```rust
# #![allow(unused_comparisons)]
use random_number::rand;

let mut thread_rng = rand::thread_rng();

let f: f64 = random_number::random!(thread_rng);

assert!(0.0 <= f && f <= 1.0);

let i: u8 = random_number::random!(.., thread_rng);

assert!(0 <= i && i <= 255);
```

```rust
# #![allow(unused_comparisons)]
let i: u8 = random_number::random!(..=10);

assert!(0 <= i && i <= 10);

let i: i8 = random_number::random!(..=0);

assert!(-128 <= i && i <= 0);
```

```rust
# #![allow(unused_comparisons)]
use random_number::rand;

let mut thread_rng = rand::thread_rng();

let i: u8 = random_number::random!(..=10, thread_rng);

assert!(0 <= i && i <= 10);

let i: i8 = random_number::random!(..=0, thread_rng);

assert!(-128 <= i && i <= 0);
```

```rust
# #![allow(unused_comparisons)]
let i: u8 = random_number::random!(10..);

assert!(10 <= i && i <= 255);

let i: i8 = random_number::random!(0..);

assert!(0 <= i && i <= 127);
```

```rust
# #![allow(unused_comparisons)]
use random_number::rand;

let mut thread_rng = rand::thread_rng();

let i: u8 = random_number::random!(10.., thread_rng);

assert!(10 <= i && i <= 255);

let i: i8 = random_number::random!(0.., thread_rng);

assert!(0 <= i && i <= 127);
```

```rust
# #![allow(unused_comparisons)]
let i: u8 = random_number::random!(..10);

assert!(0 <= i && i <= 9);

let i: i8 = random_number::random!(..0);

assert!(-128 <= i && i <= -1);
```

```rust
# #![allow(unused_comparisons)]
use random_number::rand;

let mut thread_rng = rand::thread_rng();

let i: u8 = random_number::random!(..10, thread_rng);

assert!(0 <= i && i <= 9);

let i: i8 = random_number::random!(..0, thread_rng);

assert!(-128 <= i && i <= -1);
```

```rust
let i: u8 = random_number::random!(1..10);

assert!(1 <= i && i <= 10);

let i: i8 = random_number::random!(-2..=12);

assert!(-2 <= i && i <= 12);
```

```rust
use random_number::rand;

let mut thread_rng = rand::thread_rng();

let i: u8 = random_number::random!(1..10, thread_rng);

assert!(1 <= i && i <= 10);

let i: i8 = random_number::random!(-2..=12, thread_rng);

assert!(-2 <= i && i <= 12);
```

```rust
let i: u8 = random_number::random!(12, 20);

assert!(12 <= i && i <= 20);

let i: i8 = random_number::random!(20, 12);

assert!(12 <= i && i <= 20);
```

```rust
use random_number::rand;

let mut thread_rng = rand::thread_rng();

let i: u8 = random_number::random!(12, 20, thread_rng);

assert!(12 <= i && i <= 20);

let i: i8 = random_number::random!(20, 12, thread_rng);

assert!(12 <= i && i <= 20);
```
*/
#[proc_macro_hack]
pub use random_number_macro_impl::random;
/**
Generate random numbers.

## Examples

```rust
# #![allow(unused_comparisons)]
let mut f = [0f64; 100];
random_number::random_fill!(f);

for f in f.iter().copied() {
    assert!(0.0 <= f && f <= 1.0);
}

let mut i = [0u8; 100];
random_number::random_fill!(i);

for i in i.iter().copied() {
    assert!(0 <= i && i <= 255);
}
```

```rust
# #![allow(unused_comparisons)]
use random_number::rand;

let mut thread_rng = rand::thread_rng();

let mut f = [0f64; 100];
random_number::random_fill!(f, thread_rng);

for f in f.iter().copied() {
    assert!(0.0 <= f && f <= 1.0);
}

let mut i = [0u8; 100];
random_number::random_fill!(i, thread_rng);

for i in i.iter().copied() {
    assert!(0 <= i && i <= 255);
}
```

```rust
# #![allow(unused_comparisons)]
let mut i = [0u8; 100];
random_number::random_fill!(i, ..=10);

for i in i.iter().copied() {
    assert!(0 <= i && i <= 10);
}

let mut i = [0i8; 100];
random_number::random_fill!(i, ..=0);

for i in i.iter().copied() {
    assert!(-128 <= i && i <= 0);
}
```

```rust
# #![allow(unused_comparisons)]
use random_number::rand;

let mut thread_rng = rand::thread_rng();

let mut i = [0u8; 100];
random_number::random_fill!(i, ..=10, thread_rng);

for i in i.iter().copied() {
    assert!(0 <= i && i <= 10);
}

let mut i = [0i8; 100];
random_number::random_fill!(i, ..=0, thread_rng);

for i in i.iter().copied() {
    assert!(-128 <= i && i <= 0);
}
```

```rust
# #![allow(unused_comparisons)]
let mut i = [0u8; 100];
random_number::random_fill!(i, 10..);

for i in i.iter().copied() {
    assert!(10 <= i && i <= 255);
}

let mut i = [0i8; 100];
random_number::random_fill!(i, 0..);

for i in i.iter().copied() {
    assert!(0 <= i && i <= 127);
}
```

```rust
# #![allow(unused_comparisons)]
use random_number::rand;

let mut thread_rng = rand::thread_rng();

let mut i = [0u8; 100];
random_number::random_fill!(i, 10.., thread_rng);

for i in i.iter().copied() {
    assert!(10 <= i && i <= 255);
}

let mut i = [0i8; 100];
random_number::random_fill!(i, 0.., thread_rng);

for i in i.iter().copied() {
    assert!(0 <= i && i <= 127);
}
```

```rust
# #![allow(unused_comparisons)]
let mut i = [0u8; 100];
random_number::random_fill!(i, ..10);

for i in i.iter().copied() {
    assert!(0 <= i && i <= 9);
}

let mut i = [0i8; 100];
random_number::random_fill!(i, ..0);

for i in i.iter().copied() {
    assert!(-128 <= i && i <= -1);
}
```

```rust
# #![allow(unused_comparisons)]
use random_number::rand;

let mut thread_rng = rand::thread_rng();

let mut i = [0u8; 100];
random_number::random_fill!(i, ..10, thread_rng);

for i in i.iter().copied() {
    assert!(0 <= i && i <= 9);
}

let mut i = [0i8; 100];
random_number::random_fill!(i, ..0, thread_rng);

for i in i.iter().copied() {
    assert!(-128 <= i && i <= -1);
}
```

```rust
let mut i = [0u8; 100];
random_number::random_fill!(i, 1..10);

for i in i.iter().copied() {
    assert!(1 <= i && i <= 10);
}

let mut i = [0i8; 100];
random_number::random_fill!(i, -2..12);

for i in i.iter().copied() {
    assert!(-2 <= i && i <= 12);
}
```

```rust
use random_number::rand;

let mut thread_rng = rand::thread_rng();

let mut i = [0u8; 100];
random_number::random_fill!(i, 1..10, thread_rng);

for i in i.iter().copied() {
    assert!(1 <= i && i <= 10);
}

let mut i = [0i8; 100];
random_number::random_fill!(i, -2..12, thread_rng);

for i in i.iter().copied() {
    assert!(-2 <= i && i <= 12);
}
```

```rust
let mut i = [0u8; 100];
random_number::random_fill!(i, 12, 20);

for i in i.iter().copied() {
    assert!(12 <= i && i <= 20);
}

let mut i = [0i8; 100];
random_number::random_fill!(i, 12, 20);

for i in i.iter().copied() {
    assert!(12 <= i && i <= 20);
}
```

```rust
use random_number::rand;

let mut thread_rng = rand::thread_rng();

let mut i = [0u8; 100];
random_number::random_fill!(i, 12, 20, thread_rng);

for i in i.iter().copied() {
    assert!(12 <= i && i <= 20);
}

let mut i = [0i8; 100];
random_number::random_fill!(i, 12, 20, thread_rng);

for i in i.iter().copied() {
    assert!(12 <= i && i <= 20);
}
```
*/
#[proc_macro_hack]
pub use random_number_macro_impl::random_fill;
