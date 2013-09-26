// (C)opyleft 2013 Frank Denis

/*!
 * Bloom filter for Rust
 *
 * This is a simple but fast Bloom filter implementation, that requires only
 * 2 hash functions, generated with SipHash-2-4 using randomized keys.
 */

#[link(name = "bloomfilter", vers = "0.1")];
#[crate_type = "lib"];
#[license = "BSD"];
#[warn(non_camel_case_types, non_uppercase_statics, non_uppercase_statics, unnecessary_qualification, managed_heap_memory)]

extern mod extra;

use std::hash;
use std::num;
use std::rand;
use std::rand::Rng;
use extra::bitv;

struct SipHashKey {
    k1: u64,
    k2: u64
}

impl SipHashKey {
    fn new(k1: u64, k2: u64) -> SipHashKey {
        SipHashKey {
            k1: k1,
            k2: k2
        }
    }

    fn new_random() -> SipHashKey {
        let mut rng = rand::task_rng();
        SipHashKey {
            k1: rand::Rand::rand(& mut rng),
            k2: rand::Rand::rand(& mut rng)
        }
    }
}

/// Bloom filter structure
pub struct Bloom {
    priv bitmap: bitv::Bitv,
    priv bitmap_bits: u64,
    priv k_num: uint,
    priv skeys: [SipHashKey, ..2]
}

impl Bloom {

/// Create a new bloom filter structure.
/// bitmap_size is the size in bytes (not bits) that will be allocated in memory
/// items_count is an estimation of the maximum number of items to store
    pub fn new(bitmap_size: uint, items_count: uint) -> Bloom {
        assert!(bitmap_size > 0u && items_count > 0u);
        let bitmap_bits = (bitmap_size as u64) * 8u64;
        let k_num = Bloom::optimal_k_num(bitmap_bits, items_count);
        let bitmap = bitv::Bitv::new(bitmap_bits as uint, false);
        let skeys = [ SipHashKey::new_random(), SipHashKey::new_random() ];
        Bloom {
            bitmap: bitmap,
            bitmap_bits: bitmap_bits,
            k_num: k_num,
            skeys: skeys
        }
    }

/// Record the presence of an item.
    pub fn set<T: hash::Hash>(& mut self, item: T) {
        let mut hashes = [ 0u64, 0u64 ];
        for k_i in range(0u, self.k_num) {
            let bit_offset = (self.bloom_hash(& mut hashes, &item, k_i)
                              % self.bitmap_bits) as uint;
            self.bitmap.set(bit_offset, true);
        }
    }

/// Check if an item is present in the set.
/// There can be false positives, but no false negatives.
    pub fn check<T: hash::Hash>(&self, item: T) -> bool {
        let mut hashes = [ 0u64, 0u64 ];
        for k_i in range(0u, self.k_num) {
            let bit_offset = (self.bloom_hash(& mut hashes, &item, k_i)
                              % self.bitmap_bits) as uint;
            if self.bitmap.get(bit_offset) == false {
                return false;
            }
        }
        true
    }

/// Record the presence of an item in the set,
/// and return the previous state of this item.
    pub fn check_and_set<T: hash::Hash>(&mut self, item: T) -> bool {
        let mut hashes = [ 0u64, 0u64 ];
        let mut found = true;
        for k_i in range(0u, self.k_num) {
            let bit_offset = (self.bloom_hash(& mut hashes, &item, k_i)
                              % self.bitmap_bits) as uint;
            if self.bitmap.get(bit_offset) == false {
                found = false;
                self.bitmap.set(bit_offset, true);
            }
        }
        found
    }

    fn optimal_k_num(bitmap_bits: u64, items_count: uint) -> uint {
        let m = bitmap_bits as f64;
        let n = items_count as f64;
        let k_num = (m / n * num::ln(2.0f64).ceil()) as uint;
        num::max(k_num, 1)
    }

    fn bloom_hash<T: hash::Hash>(&self, hashes: & mut [u64, ..2],
                  item: &T, k_i: uint) -> u64 {
        if k_i < 2 {
            let skey = self.skeys[k_i];
            let hash = (*item).hash_keyed(skey.k1, skey.k2);
            hashes[k_i] = hash;
            hash
        } else {
            hashes[0] + (((k_i as u64) * hashes[1]) % 0xffffffffffffffc5)
        }
    }
}

#[test]
fn bloom_test_set() {
    let mut bloom = Bloom::new(10, 80);
    let key = &rand::task_rng().gen_ascii_str(16u);
    assert!(bloom.check(key) == false);
    bloom.set(&key);
    assert!(bloom.check(key.clone()) == true);
}

#[test]
fn bloom_test_check_and_set() {
    let mut bloom = Bloom::new(10, 80);
    let key = &rand::task_rng().gen_ascii_str(16u);
    assert!(bloom.check_and_set(key) == false);
    assert!(bloom.check_and_set(key.clone()) == true);
}
