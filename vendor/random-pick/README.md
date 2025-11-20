Random Pick
====================

[![CI](https://github.com/magiclen/random-pick/actions/workflows/ci.yml/badge.svg)](https://github.com/magiclen/random-pick/actions/workflows/ci.yml)

Pick an element from a slice randomly by given weights.

## Example

```rust
enum Prize {
    Legendary,
    Rare,
    Enchanted,
    Common,
}

let prize_list = [Prize::Legendary, Prize::Rare, Prize::Enchanted, Prize::Common]; // available prizes

let slice = &prize_list;
let weights = [1, 5, 15, 30]; // a scale of chance of picking each kind of prize

let n = 1000000;
let mut counter = [0usize; 4];

for _ in 0..n {
    let picked_item = random_pick::pick_from_slice(slice, &weights).unwrap();

    match picked_item {
        Prize::Legendary=>{
            counter[0] += 1;
           }
        Prize::Rare=>{
            counter[1] += 1;
        }
        Prize::Enchanted=>{
            counter[2] += 1;
        }
        Prize::Common=>{
            counter[3] += 1;
        }
    }
}

println!("{}", counter[0]); // Should be close to 20000
println!("{}", counter[1]); // Should be close to 100000
println!("{}", counter[2]); // Should be close to 300000
println!("{}", counter[3]); // Should be close to 600000
```

The length of the slice is usually an integral multiple (larger than zero) of that of weights.

If you have multiple slices, you don't need to use extra space to concat them, just use the `pick_from_multiple_slices` function, instead of `pick_from_slice`.

Besides picking a single element from a slice or slices, you can also use `pick_multiple_from_slice` and `pick_multiple_from_multiple_slices` functions. Their overhead is lower than that of non-multiple-pick functions with extra loops.

## Crates.io

https://crates.io/crates/random-pick

## Documentation

https://docs.rs/random-pick

## License

[MIT](LICENSE)