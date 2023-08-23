// (C)opyleft 2013-2021 Frank Denis
// Licensed under the ICS license (https://opensource.org/licenses/ISC)

//! Bloom filter for Rust
//!
//! This is a simple but fast Bloom filter implementation, that requires only
//! 2 hash functions, generated with SipHash-1-3 using randomized keys.

#![warn(non_camel_case_types, non_upper_case_globals, unused_qualifications)]
#![allow(clippy::unreadable_literal, clippy::bool_comparison)]

use std::cmp;
use std::convert::TryFrom;
use std::f64;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

use bit_vec::BitVec;
#[cfg(feature = "random")]
use getrandom::getrandom;
#[cfg(feature = "serde")]
use siphasher::reexports::serde;
use siphasher::sip::SipHasher13;

pub mod reexports {
    #[cfg(feature = "random")]
    pub use ::getrandom;
    pub use bit_vec;
    pub use siphasher;
    #[cfg(feature = "serde")]
    pub use siphasher::reexports::serde;
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
    /// bitmap_size is the size in bytes (not bits) that will be allocated in
    /// memory items_count is an estimation of the maximum number of items
    /// to store. seed is a random value used to generate the hash
    /// functions.
    pub fn new_with_seed(bitmap_size: usize, items_count: usize, seed: &[u8; 32]) -> Self {
        assert!(bitmap_size > 0 && items_count > 0);
        let bitmap_bits = u64::try_from(bitmap_size)
            .unwrap()
            .checked_mul(8u64)
            .unwrap();
        let k_num = Self::optimal_k_num(bitmap_bits, items_count);
        let bitmap = BitVec::from_elem(usize::try_from(bitmap_bits).unwrap(), false);
        let mut k1 = [0u8; 16];
        let mut k2 = [0u8; 16];
        k1.copy_from_slice(&seed[0..16]);
        k2.copy_from_slice(&seed[16..32]);
        let sips = [Self::sip_new(&k1), Self::sip_new(&k2)];
        Self {
            bit_vec: bitmap,
            bitmap_bits,
            k_num,
            sips,
            _phantom: PhantomData,
        }
    }

    /// Create a new bloom filter structure.
    /// bitmap_size is the size in bytes (not bits) that will be allocated in
    /// memory items_count is an estimation of the maximum number of items
    /// to store.
    #[cfg(feature = "random")]
    pub fn new(bitmap_size: usize, items_count: usize) -> Self {
        let mut seed = [0u8; 32];
        getrandom(&mut seed).unwrap();
        Self::new_with_seed(bitmap_size, items_count, &seed)
    }

    /// Create a new bloom filter structure.
    /// items_count is an estimation of the maximum number of items to store.
    /// fp_p is the wanted rate of false positives, in ]0.0, 1.0[
    #[cfg(feature = "random")]
    pub fn new_for_fp_rate(items_count: usize, fp_p: f64) -> Self {
        let bitmap_size = Self::compute_bitmap_size(items_count, fp_p);
        Bloom::new(bitmap_size, items_count)
    }

    /// Create a new bloom filter structure.
    /// items_count is an estimation of the maximum number of items to store.
    /// fp_p is the wanted rate of false positives, in ]0.0, 1.0[
    pub fn new_for_fp_rate_with_seed(items_count: usize, fp_p: f64, seed: &[u8; 32]) -> Self {
        let bitmap_size = Self::compute_bitmap_size(items_count, fp_p);
        Bloom::new_with_seed(bitmap_size, items_count, seed)
    }

    /// Create a bloom filter structure from a previous state given as a
    /// `ByteVec` structure. The state is assumed to be retrieved from an
    /// existing bloom filter.
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

    /// Create a bloom filter structure with an existing state given as a byte
    /// array. The state is assumed to be retrieved from an existing bloom
    /// filter.
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

    #[allow(dead_code)]
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
            (hashes[0]).wrapping_add((k_i as u64).wrapping_mul(hashes[1]))
                % 0xFFFF_FFFF_FFFF_FFC5u64 //largest u64 prime
        }
    }

    /// Clear all of the bits in the filter, removing all keys from the set
    pub fn clear(&mut self) {
        self.bit_vec.clear()
    }

    /// Set all of the bits in the filter, making it appear like every key is in the set
    pub fn fill(&mut self) {
        self.bit_vec.set_all()
    }

    /// Test if there are no elements in the set
    pub fn is_empty(&self) -> bool {
        !self.bit_vec.any()
    }

    #[inline]
    fn sip_new(key: &[u8; 16]) -> SipHasher13 {
        SipHasher13::new_with_key(key)
    }
}
