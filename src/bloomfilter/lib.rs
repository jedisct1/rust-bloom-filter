// (C)opyleft 2013,2014 Frank Denis

/*!
 * Bloom filter for Rust
 *
 * This is a simple but fast Bloom filter implementation, that requires only
 * 2 hash functions, generated with SipHash-2-4 using randomized keys.
 */

#![desc = "A fast Bloom filter implementation."]
#![license = "BSD"]
#![crate_name="bloomfilter"]
#![crate_type = "rlib"]
#![warn(non_camel_case_types, non_upper_case_globals, unused_qualifications)]

extern crate collections;

use std::cmp;
use std::f64;
use std::hash::Hash;
use std::hash::sip::SipHasher;
use std::hash::Hasher;
use std::rand;
use collections::bitv;

#[cfg(test)]
use std::rand::Rng;

/// Bloom filter structure
pub struct Bloom {
    bitmap: bitv::Bitv,
    bitmap_bits: u64,
    k_num: uint,
    sips: [SipHasher, ..2]
}

impl Bloom {
/// Create a new bloom filter structure.
/// bitmap_size is the size in bytes (not bits) that will be allocated in memory
/// items_count is an estimation of the maximum number of items to store.
    pub fn new(bitmap_size: uint, items_count: uint) -> Bloom {
        assert!(bitmap_size > 0u && items_count > 0u);
        let bitmap_bits = (bitmap_size as u64) * 8u64;
        let k_num = Bloom::optimal_k_num(bitmap_bits, items_count);
        let bitmap = bitv::Bitv::with_capacity(bitmap_bits as uint, false);
        let sips = [ Bloom::sip_new(), Bloom::sip_new() ];
        Bloom {
            bitmap: bitmap,
            bitmap_bits: bitmap_bits,
            k_num: k_num,
            sips: sips
        }
    }

/// Create a new bloom filter structure.
/// items_count is an estimation of the maximum number of items to store.
/// fp_p is the wanted rate of false positives, in ]0.0, 1.0[
    pub fn new_for_fp_rate(items_count: uint, fp_p: f64) -> Bloom {
        let bitmap_size = Bloom::compute_bitmap_size(items_count, fp_p);
        Bloom::new(bitmap_size, items_count)
    }

/// Compute a recommended bitmap size for items_count items
/// and a fp_p rate of false positives.
/// fp_p obviously has to be within the ]0.0, 1.0[ range.
    pub fn compute_bitmap_size(items_count: uint, fp_p: f64) -> uint {
        assert!(items_count > 0u);
        assert!(fp_p > 0.0 && fp_p < 1.0);
        let log2 = f64::consts::LN_2;
        let log2_2 = log2 * log2;
        ((items_count as f64) * fp_p.ln() / (-8.0 * log2_2)).ceil() as uint
    }

/// Record the presence of an item.
    pub fn set<T: Hash>(& mut self, item: T) {
        let mut hashes = [ 0u64, 0u64 ];
        for k_i in range(0u, self.k_num) {
            let bit_offset = (self.bloom_hash(& mut hashes, &item, k_i)
                              % self.bitmap_bits) as uint;
            self.bitmap.set(bit_offset, true);
        }
    }

/// Check if an item is present in the set.
/// There can be false positives, but no false negatives.
    pub fn check<T: Hash>(&self, item: T) -> bool {
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
    pub fn check_and_set<T: Hash>(&mut self, item: T) -> bool {
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

/// Return the number of bits in the filter
    pub fn number_of_bits(&self) -> u64 {
        self.bitmap_bits
    }

/// Return the number of hash functions used for `check` and `set` 
    pub fn number_of_hash_functions(&self) -> uint {
        self.k_num
    }

    fn optimal_k_num(bitmap_bits: u64, items_count: uint) -> uint {
        let m = bitmap_bits as f64;
        let n = items_count as f64;
        let k_num = (m / n * 2.0f64.ln()).ceil() as uint;
        cmp::max(k_num, 1)
    }

    fn bloom_hash<T: Hash>(&self, hashes: & mut [u64, ..2],
                  item: &T, k_i: uint) -> u64 {
        if k_i < 2 {
            let sip = self.sips[k_i];
            let hash = sip.hash(item);
            hashes[k_i] = hash;
            hash
        } else {
            hashes[0] + (((k_i as u64) * hashes[1]) % 0xffffffffffffffc5)
        }
    }

    fn sip_new() -> SipHasher {
        let mut rng = rand::task_rng();
        SipHasher::new_with_keys(rand::Rand::rand(& mut rng),
                                 rand::Rand::rand(& mut rng))
    }
}

#[test]
fn bloom_test_set() {
    let mut bloom = Bloom::new(10, 80);
    let key: &Vec<u8> = &rand::task_rng().gen_iter::<u8>().take(16u).collect();
    assert!(bloom.check(key) == false);
    bloom.set(&key);
    assert!(bloom.check(key.clone()) == true);
}

#[test]
fn bloom_test_check_and_set() {
    let mut bloom = Bloom::new(10, 80);
    let key: &Vec<u8> = &rand::task_rng().gen_iter::<u8>().take(16u).collect();
    assert!(bloom.check_and_set(key) == false);
    assert!(bloom.check_and_set(key.clone()) == true);
}
