# bloomfilter <img src="img/logo.png" align="right" width="150" height="auto" />
[![Crates.io](https://img.shields.io/crates/v/bloomfilter.svg)](https://crates.io/crates/bloomfilter)
<a href="https://github.com/jedisct1/rust-bloom-filter/actions?query=workflow%3A%22Continuous+Integration%22">
    <img src="https://img.shields.io/github/workflow/status/jedisct1/rust-bloom-filter/Continuous%20Integration?style=flat&logo=GitHub%20Actions">
</a>
[![docs.rs](https://docs.rs/bloomfilter/badge.svg)](https://docs.rs/bloomfilter)
[![Rust](https://img.shields.io/badge/rust-1.38.0%2B-blue.svg?maxAge=3600)](https://github.com/jedisct1/rust-bloom-filter)
[![License: ISC](https://img.shields.io/badge/License-ISC-blue.svg)](https://github.com/jedisct1/rust-bloom-filter/blob/master/LICENSE)
<a href="https://codecov.io/gh/Ebedthan/rust-bloom-filter">
    <img src="https://codecov.io/gh/Ebedthan/rust-bloom-filter/branch/main/graph/badge.svg">
</a>
 

A simple but fast implementation of the Bloom filter in Rust. The Bloom filter is a a space-efficient probabilistic data structure supporting dynamic set membership queries with false positives. It was introduced by Burton H. Bloom in 1970 [(Bloom, 1970)](https://dl.acm.org/doi/10.1145/362686.362692) and have since benn increasingly used in computing applications and bioinformatics.

### Documentation

Library documentation with examples is available on [docs.rs](https://docs.rs/bloomfilter).


### Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
bloomfilter = "1"
```

Here is a simple example for creating a bloom filter with a false positive rate of 0.001 and query for presence of some numbers.

```rust
use bloomfilter::Bloom;

let num_items = 100000;
let fp_rate = 0.001;

let mut bloom = Bloom::new_for_fp_rate(num_items, fp_rate);
bloom.set(&10);   // insert 10 in the bloom filter
bloom.check(&10); // return true
bloom.check(&20); // return false
```

### Minimum Rust version
This crate's minimum supported `rustc` version is 1.38.0.

### License
This project is licensed under the ISC license ([LICENSE](https://github.com/jedisct1/rust-bloom-filter/blob/master/LICENSE) or https://opensource.org/licenses/ISC).