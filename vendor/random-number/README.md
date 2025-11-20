Random Number
====================

[![CI](https://github.com/magiclen/random-number/actions/workflows/ci.yml/badge.svg)](https://github.com/magiclen/random-number/actions/workflows/ci.yml)

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

```rust
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

## Crates.io

https://crates.io/crates/random-number

## Documentation

https://docs.rs/random-number

## License

[MIT](LICENSE)
