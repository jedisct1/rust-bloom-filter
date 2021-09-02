// (C)opyleft 2013-2021 Frank Denis

//! Bloom filter for Rust
//!
//! This is a simple but fast Bloom filter implementation, that requires only
//! 2 hash functions, generated with SipHash-1-3 using randomized keys.
//!

#![warn(non_camel_case_types, non_upper_case_globals, unused_qualifications)]
#![allow(clippy::unreadable_literal, clippy::bool_comparison)]

use bit_vec::BitVec;
use rand::prelude::*;
use siphasher::sip::SipHasher13;
use std::cmp;
use std::f64;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

#[cfg(feature = "serde")]
use siphasher::reexports::serde;

#[cfg(test)]
use rand::Rng;

pub mod reexports {
    pub use bit_vec;
    pub use rand;
    #[cfg(feature = "serde")]
    pub use serde;
    pub use siphasher;
}

/// Bloom filter structure
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(crate = "serde"))]
#[derive(Clone, Debug)]
pub struct Bloom<T: ?Sized> {
    bit_vec: BitVec,
    bitmap_bits: u64,
    k_num: u32,
    sips: [SipHasher13; 2],

    _phantom: PhantomData<T>,
}

impl<T: ?Sized> Bloom<T> {
    /// Create a new bloom filter structure.
    /// bitmap_size is the size in bytes (not bits) that will be allocated in memory
    /// items_count is an estimation of the maximum number of items to store.
    pub fn new(bitmap_size: usize, items_count: usize) -> Self {
        assert!(bitmap_size > 0 && items_count > 0);
        let bitmap_bits = (bitmap_size as u64) * 8u64;
        let k_num = Self::optimal_k_num(bitmap_bits, items_count);
        let bitmap = BitVec::from_elem(bitmap_bits as usize, false);
        let sips = [Self::sip_new(), Self::sip_new()];
        Self {
            bit_vec: bitmap,
            bitmap_bits,
            k_num,
            sips,
            _phantom: PhantomData,
        }
    }

    /// Create a new bloom filter structure.
    /// items_count is an estimation of the maximum number of items to store.
    /// fp_p is the wanted rate of false positives, in ]0.0, 1.0[
    pub fn new_for_fp_rate(items_count: usize, fp_p: f64) -> Self {
        let bitmap_size = Self::compute_bitmap_size(items_count, fp_p);
        Bloom::new(bitmap_size, items_count)
    }

    /// Create a bloom filter structure from a previous state given as a `ByteVec` structure.
    /// The state is assumed to be retrieved from an existing bloom filter.
    pub fn from_bit_vec(
        bit_vec: BitVec,
        bitmap_bits: u64,
        k_num: u32,
        sip_keys: [(u64, u64); 2],
    ) -> Self {
        let sips = [
            SipHasher13::new_with_keys(sip_keys[0].0, sip_keys[0].1),
            SipHasher13::new_with_keys(sip_keys[1].0, sip_keys[1].1),
        ];
        Self {
            bit_vec,
            bitmap_bits,
            k_num,
            sips,
            _phantom: PhantomData,
        }
    }

    /// Create a bloom filter structure with an existing state given as a byte array.
    /// The state is assumed to be retrieved from an existing bloom filter.
    pub fn from_existing(
        bytes: &[u8],
        bitmap_bits: u64,
        k_num: u32,
        sip_keys: [(u64, u64); 2],
    ) -> Self {
        Self::from_bit_vec(BitVec::from_bytes(bytes), bitmap_bits, k_num, sip_keys)
    }

    /// Compute a recommended bitmap size for items_count items
    /// and a fp_p rate of false positives.
    /// fp_p obviously has to be within the ]0.0, 1.0[ range.
    pub fn compute_bitmap_size(items_count: usize, fp_p: f64) -> usize {
        assert!(items_count > 0);
        assert!(fp_p > 0.0 && fp_p < 1.0);
        let log2 = f64::consts::LN_2;
        let log2_2 = log2 * log2;
        ((items_count as f64) * f64::ln(fp_p) / (-8.0 * log2_2)).ceil() as usize
    }

    /// Record the presence of an item.
    pub fn set(&mut self, item: &T)
    where
        T: Hash,
    {
        let mut hashes = [0u64, 0u64];
        for k_i in 0..self.k_num {
            let bit_offset = (self.bloom_hash(&mut hashes, item, k_i) % self.bitmap_bits) as usize;
            self.bit_vec.set(bit_offset, true);
        }
    }

    /// Check if an item is present in the set.
    /// There can be false positives, but no false negatives.
    pub fn check(&self, item: &T) -> bool
    where
        T: Hash,
    {
        let mut hashes = [0u64, 0u64];
        for k_i in 0..self.k_num {
            let bit_offset = (self.bloom_hash(&mut hashes, item, k_i) % self.bitmap_bits) as usize;
            if self.bit_vec.get(bit_offset).unwrap() == false {
                return false;
            }
        }
        true
    }

    /// Record the presence of an item in the set,
    /// and return the previous state of this item.
    pub fn check_and_set(&mut self, item: &T) -> bool
    where
        T: Hash,
    {
        let mut hashes = [0u64, 0u64];
        let mut found = true;
        for k_i in 0..self.k_num {
            let bit_offset = (self.bloom_hash(&mut hashes, item, k_i) % self.bitmap_bits) as usize;
            if self.bit_vec.get(bit_offset).unwrap() == false {
                found = false;
                self.bit_vec.set(bit_offset, true);
            }
        }
        found
    }

    /// Return the bitmap as a vector of bytes
    pub fn bitmap(&self) -> Vec<u8> {
        self.bit_vec.to_bytes()
    }

    /// Return the bitmap as a "BitVec" structure
    pub fn bit_vec(&self) -> &BitVec {
        &self.bit_vec
    }

    /// Return the number of bits in the filter
    pub fn number_of_bits(&self) -> u64 {
        self.bitmap_bits
    }

    /// Return the number of hash functions used for `check` and `set`
    pub fn number_of_hash_functions(&self) -> u32 {
        self.k_num
    }

    /// Return the keys used by the sip hasher
    pub fn sip_keys(&self) -> [(u64, u64); 2] {
        [self.sips[0].keys(), self.sips[1].keys()]
    }

    fn optimal_k_num(bitmap_bits: u64, items_count: usize) -> u32 {
        let m = bitmap_bits as f64;
        let n = items_count as f64;
        let k_num = (m / n * f64::ln(2.0f64)).ceil() as u32;
        cmp::max(k_num, 1)
    }

    fn bloom_hash(&self, hashes: &mut [u64; 2], item: &T, k_i: u32) -> u64
    where
        T: Hash,
    {
        if k_i < 2 {
            let sip = &mut self.sips[k_i as usize].clone();
            item.hash(sip);
            let hash = sip.finish();
            hashes[k_i as usize] = hash;
            hash
        } else {
            (hashes[0] as u128).wrapping_add((k_i as u128).wrapping_mul(hashes[1] as u128)) as u64
                % 0xffffffffffffffc5
        }
    }

    /// Clear all of the bits in the filter, removing all keys from the set
    pub fn clear(&mut self) {
        self.bit_vec.clear()
    }

    fn sip_new() -> SipHasher13 {
        let mut rng = thread_rng();
        SipHasher13::new_with_keys(rng.gen(), rng.gen())
    }
}

#[test]
fn bloom_test_set() {
    let mut rng = thread_rng();
    let mut bloom = Bloom::new(10, 80);
    let mut key = vec![0u8, 16];
    rng.fill_bytes(&mut key);
    assert!(bloom.check(&key) == false);
    bloom.set(&key);
    assert!(bloom.check(&key) == true);
}

#[test]
fn bloom_test_check_and_set() {
    let mut rng = thread_rng();
    let mut bloom = Bloom::new(10, 80);
    let mut key = vec![0u8, 16];
    rng.fill_bytes(&mut key);
    assert!(bloom.check_and_set(&key) == false);
    assert!(bloom.check_and_set(&key) == true);
}

#[test]
fn bloom_test_clear() {
    let mut rng = thread_rng();
    let mut bloom = Bloom::new(10, 80);
    let mut key = vec![0u8, 16];
    rng.fill_bytes(&mut key);
    bloom.set(&key);
    assert!(bloom.check(&key) == true);
    bloom.clear();
    assert!(bloom.check(&key) == false);
}

#[test]
fn bloom_test_load() {
    let mut rng = thread_rng();
    let mut original = Bloom::new(10, 80);
    let mut key = vec![0u8, 16];
    rng.fill_bytes(&mut key);
    original.set(&key);
    assert!(original.check(&key) == true);

    let cloned = Bloom::from_existing(
        &original.bitmap(),
        original.number_of_bits(),
        original.number_of_hash_functions(),
        original.sip_keys(),
    );
    assert!(cloned.check(&key) == true);
}
