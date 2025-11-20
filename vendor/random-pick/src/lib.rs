/*!
# Random Pick
Pick an element from a slice randomly by given weights.

## Examples

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
*/

use random_number::rand::thread_rng;
use random_number::random;

const MAX_NUMBER: usize = usize::MAX;

/// Pick an element from a slice randomly by given weights.
pub fn pick_from_slice<'a, T>(slice: &'a [T], weights: &'a [usize]) -> Option<&'a T> {
    let slice_len = slice.len();

    let index = gen_usize_with_weights(slice_len, weights)?;

    Some(&slice[index])
}

/// Pick an element from multiple slices randomly by given weights.
pub fn pick_from_multiple_slices<'a, T>(slices: &[&'a [T]], weights: &'a [usize]) -> Option<&'a T> {
    let len: usize = slices.iter().map(|slice| slice.len()).sum();

    let mut index = gen_usize_with_weights(len, weights)?;

    for slice in slices {
        let len = slice.len();

        if index < len {
            return Some(&slice[index]);
        } else {
            index -= len;
        }
    }

    None
}

/// Pick multiple elements from a slice randomly by given weights.
pub fn pick_multiple_from_slice<'a, T>(
    slice: &'a [T],
    weights: &'a [usize],
    count: usize,
) -> Vec<&'a T> {
    let slice_len = slice.len();

    gen_multiple_usize_with_weights(slice_len, weights, count)
        .iter()
        .map(|&index| &slice[index])
        .collect()
}

/// Pick multiple elements from multiple slices randomly by given weights.
pub fn pick_multiple_from_multiple_slices<'a, T>(
    slices: &[&'a [T]],
    weights: &'a [usize],
    count: usize,
) -> Vec<&'a T> {
    let len: usize = slices.iter().map(|slice| slice.len()).sum();

    gen_multiple_usize_with_weights(len, weights, count)
        .iter()
        .map(|index| {
            let mut index = *index;

            let mut s = slices[0];

            for slice in slices {
                let len = slice.len();

                if index < len {
                    s = slice;
                    break;
                } else {
                    index -= len;
                }
            }

            &s[index]
        })
        .collect()
}

/// Get a usize value by given weights.
pub fn gen_usize_with_weights(high: usize, weights: &[usize]) -> Option<usize> {
    let weights_len = weights.len();

    if weights_len == 0 || high == 0 {
        return None;
    } else if weights_len == 1 {
        if weights[0] == 0 {
            return None;
        }

        return Some(random!(0..high));
    } else {
        let mut weights_sum = 0f64;
        let mut max_weight = 0;

        for w in weights.iter().copied() {
            weights_sum += w as f64;
            if w > max_weight {
                max_weight = w;
            }
        }

        if max_weight == 0 {
            return None;
        }

        let mut rng = thread_rng();

        let index_scale = (high as f64) / (weights_len as f64);

        let weights_scale = (MAX_NUMBER as f64) / weights_sum;

        let rnd = random!(0..=MAX_NUMBER, rng) as f64;

        let mut temp = 0f64;

        for (i, w) in weights.iter().copied().enumerate() {
            temp += (w as f64) * weights_scale;
            if temp > rnd {
                let index = ((i as f64) * index_scale) as usize;

                return Some(random!(index..((((i + 1) as f64) * index_scale) as usize), rng));
            }
        }
    }

    None
}

/// Get multiple usize values by given weights.
pub fn gen_multiple_usize_with_weights(high: usize, weights: &[usize], count: usize) -> Vec<usize> {
    let mut result: Vec<usize> = Vec::with_capacity(count);

    let weights_len = weights.len();

    if weights_len > 0 && high > 0 {
        if weights_len == 1 {
            if weights[0] != 0 {
                let mut rng = thread_rng();

                for _ in 0..count {
                    result.push(random!(0..high, rng));
                }
            }
        } else {
            let mut weights_sum = 0f64;
            let mut max_weight = 0;

            for w in weights.iter().copied() {
                weights_sum += w as f64;
                if w > max_weight {
                    max_weight = w;
                }
            }

            if max_weight > 0 {
                let index_scale = (high as f64) / (weights_len as f64);

                let weights_scale = (MAX_NUMBER as f64) / weights_sum;

                let mut rng = thread_rng();

                for _ in 0..count {
                    let rnd = random!(0..=MAX_NUMBER, rng) as f64;

                    let mut temp = 0f64;

                    for (i, w) in weights.iter().copied().enumerate() {
                        temp += (w as f64) * weights_scale;
                        if temp > rnd {
                            let index = ((i as f64) * index_scale) as usize;

                            result.push(random!(
                                index..((((i + 1) as f64) * index_scale) as usize),
                                rng
                            ));
                            break;
                        }
                    }
                }
            }
        }
    }

    result
}
