# bench Code Documentation

**Generated on:** 2026-01-13 15:20:01
**Directory:** /Users/Shared/gnostr-org/.github/gnostr/app/db/bench
**Files included:** 4

---

## Directory Structure

```
./.gitignore
./Cargo.toml
./bench.md
./benches/event.rs
./benches/sort.rs
./src/lib.rs
```

---

## File Contents

### Cargo.toml

**Size:** 415 bytes | **Modified:** 2025-12-02 17:35:50

```toml
[package]
name = "nostr-db-bench"
version = "1905.926180.332424"
edition = "2021"
publish = false

[dependencies]
criterion = "0.5.1"
rand = "0.8.5"
rayon = "1.10.0"
anyhow = "1.0.86"
colored = "2.1.0"
tempfile = "3.12.0"
nostr-db = { path = "../"}
nostr-kv-bench = { path = "../../kv/bench"}
charabia = "0.9.0"
twox-hash = "1.6.3"

[[bench]]
name = "sort"
harness = false

[[bench]]
name = "event"
harness = false
```

---

### benches/event.rs

**Size:** 3013 bytes | **Modified:** 2025-11-22 21:18:16

```rust
use charabia::{Segment, Tokenize};
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use nostr_db::{Event, EventIndex};
use std::{hash::Hasher, str::FromStr, time::Duration};
use twox_hash::XxHash32;

fn bench_event(c: &mut Criterion) {
    let mut group = c.benchmark_group("event");
    group.measurement_time(Duration::from_secs(1));
    group.sample_size(50);
    group.warm_up_time(Duration::from_millis(100));
    group.throughput(Throughput::Elements(1));

    let json = r###"
    {"content":"Just a resource.中文 I own it, I’ve skimmed it. I’ve read it. I think it’s complete. But I have yet to apply it. No livestock on my property yet. \n\nhttps://a.co/d/fBD7pnc","created_at":1682257408,"id":"d877f51f90134aa0ee5572b393a90126e45f00ddc72242b0f9b47e90f864748c","kind":1,"pubkey":"0cf08d280aa5fcfaf340c269abcf66357526fdc90b94b3e9ff6d347a41f090b7","sig":"7d62ec09612b3e303eb8d105a5c99b2a9df6f5497b14465c235b58db2b0db8d834ee320c6d9ede8722773cddfea926a7fa108b1c829ce2208c773ba8aa44d396","tags":[["e","180f289555764f435ab5529f384fb13a79fc8df737c1b661dbaa966195636ff0"],["p","fc87ad313d6dc741dbed5a89720a7e20000b672dba0a901d9620da4c202242dd"]]}
    "###;
    let event = Event::from_str(json).unwrap();

    group.bench_function("content token", |b| {
        b.iter(|| {
            let s: &str = event.content().as_ref();
            let tokens = s.tokenize();
            for t in tokens {
                black_box(t.lemma());
            }
        })
    });
    group.bench_function("content segment", |b| {
        b.iter(|| {
            let s: &str = event.content().as_ref();
            let tokens = s.segment();
            for t in tokens {
                black_box(t.lemma());
            }
        })
    });
    group.bench_function("content segment with hash", |b| {
        b.iter(|| {
            let s: &str = event.content().as_ref();
            let tokens = s.segment();
            for t in tokens {
                let mut hasher = XxHash32::with_seed(0);
                hasher.write(t.lemma().as_bytes());
                black_box(hasher.finish() as u32);
            }
        })
    });

    group.bench_function("from_str", |b| {
        b.iter(|| black_box(Event::from_str(json).unwrap()))
    });

    group.bench_function("to_str", |b| b.iter(|| black_box(event.to_string())));

    let index_event = event.index();
    let index_bytes = event.index().to_bytes().unwrap();
    // println!("index bytes len: {}", index_bytes.len());
    group.bench_function("index_to_bytes", |b| {
        b.iter(|| black_box(index_event.to_bytes().unwrap()))
    });

    group.bench_function("index_from_bytes", |b| {
        b.iter(|| black_box(EventIndex::from_bytes(&index_bytes).unwrap()))
    });

    group.bench_function("from_zeroes", |b| {
        let e = EventIndex::from_zeroes(&index_bytes).unwrap();
        b.iter(|| black_box(e.id()))
    });
    group.finish();
}

criterion_group!(benches, bench_event);
criterion_main!(benches);
```

---

### benches/sort.rs

**Size:** 3810 bytes | **Modified:** 2025-11-22 21:18:16

```rust
use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use nostr_kv_bench::gen_pairs;
use rand::Rng;
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

fn bench_sort(c: &mut Criterion) {
    let mut group = c.benchmark_group("sort");
    group.measurement_time(Duration::from_secs(1));
    group.sample_size(50);
    group.warm_up_time(Duration::from_millis(100));
    group.throughput(Throughput::Elements(1));

    let mut rng = rand::thread_rng();

    group.bench_function("gen", |b| {
        b.iter(|| black_box(rng.gen::<u64>().to_be_bytes().to_vec()))
    });

    let pairs = gen_pairs(33, 33, 1000);

    group.bench_function("clone", |b| b.iter(|| black_box(pairs.clone())));

    group.bench_function("sort", |b| {
        b.iter(|| black_box(pairs.clone().sort_by(|a, b| a.0.cmp(&b.0))))
    });
    group.bench_function("max", |b| {
        b.iter(|| black_box(pairs.iter().max_by(|a, b| a.0.cmp(&b.0))))
    });

    let mut list = (0..3000)
        .map(|_| rng.gen::<u64>().to_be_bytes().to_vec())
        .collect::<Vec<_>>();
    list.sort();

    group.bench_function("sorted ver with pop last", |b| {
        b.iter(|| {
            let el = rng.gen::<u64>().to_be_bytes().to_vec();
            let insert_at = match list.binary_search_by(|p| p.cmp(&el)) {
                Ok(insert_at) | Err(insert_at) => insert_at,
            };
            list.insert(insert_at, el);
            black_box(list.pop().unwrap());
        })
    });
    group.bench_function("sorted ver with remove first", |b| {
        b.iter_batched(
            || rng.gen::<u64>().to_be_bytes().to_vec(),
            |el| {
                let insert_at = match list.binary_search_by(|p| p.cmp(&el)) {
                    Ok(insert_at) | Err(insert_at) => insert_at,
                };
                list.insert(insert_at, el);
                black_box(list.remove(0));
            },
            BatchSize::SmallInput,
        );
        // b.iter(|| {
        //     let el: Vec<u8> = rng.gen::<u64>().to_be_bytes().to_vec();

        // })
    });

    let mut list = VecDeque::from(list);
    group.bench_function("sorted deque with pop last", |b| {
        b.iter(|| {
            let el = rng.gen::<u64>().to_be_bytes().to_vec();
            let insert_at = match list.binary_search_by(|p| p.cmp(&el)) {
                Ok(insert_at) | Err(insert_at) => insert_at,
            };
            list.insert(insert_at, el);
            black_box(list.pop_back().unwrap());
        })
    });
    group.bench_function("sorted deque with remove first", |b| {
        b.iter(|| {
            let el = rng.gen::<u64>().to_be_bytes().to_vec();
            let insert_at = match list.binary_search_by(|p| p.cmp(&el)) {
                Ok(insert_at) | Err(insert_at) => insert_at,
            };
            list.insert(insert_at, el);
            black_box(list.pop_front().unwrap());
        })
    });

    let mut list = (0..3000)
        .map(|_| rng.gen::<u64>().to_be_bytes().to_vec())
        .collect::<Vec<_>>();
    list.sort();
    group.bench_function("custom bench - sorted ver with pop last", |b| {
        b.iter_custom(|iters| {
            let els = (0..iters)
                .map(|_| rng.gen::<u64>().to_be_bytes().to_vec())
                .collect::<Vec<_>>();
            let start = Instant::now();
            for el in els {
                let insert_at = match list.binary_search_by(|p| p.cmp(&el)) {
                    Ok(insert_at) | Err(insert_at) => insert_at,
                };
                list.insert(insert_at, el);
                black_box(list.pop().unwrap());
            }
            start.elapsed()
        });
    });
    group.finish();
}

criterion_group!(benches, bench_sort);
criterion_main!(benches);
```

---

### src/lib.rs

**Size:** 1 bytes | **Modified:** 2025-11-22 21:18:16

```rust

```

---


---
*Generated by code2prompt.sh on 2026-01-13 15:20:01*
