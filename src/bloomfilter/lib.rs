// (C)opyleft 2013-2017 Frank Denis

//! Bloom filter for Rust
//!
//! This is a simple but fast Bloom filter implementation, that requires only
//! 2 hash functions, generated with SipHash-1-3 using randomized keys.
//!

#![crate_name="bloomfilter"]
#![crate_type = "rlib"]
#![warn(non_camel_case_types, non_upper_case_globals, unused_qualifications)]

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate bit_vec;
extern crate rand;
extern crate siphasher;

use bit_vec::BitVec;
use siphasher::sip::SipHasher13;

use std::cmp;
use std::f64;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

#[cfg(test)]
use rand::Rng;

/// Bloom filter structure
#[derive(Clone)]
pub struct Bloom<T: Hash> {
    state: BloomState,
    config: BloomHasher<T>,

    // We use T for type safety but we don't use T for anything else, so we need
    // to convince the compiler that we use it
    phantom: PhantomData<T>,
}

/// The current state of a Bloom filter
#[derive(Clone)]
pub struct BloomState {
    pub(crate) bitmap: BitVec,
}

/// The configuration used to create a Bloom filter
pub struct BloomHasher<T: Hash> {
    bitmap_bits: u64,
    k_num: u32,
    sips: [SipHasher13; 2],

    // we only use T for type safety so we we don't use any methods of, but we
    // need to convince the compiler that we use it
    phantom: PhantomData<T>,
}

impl<T: Hash> Bloom<T> {
    /// Create a new bloom filter structure.
    ///
    /// * `bitmap_size` - the size in bytes (not bits) that will be allocated in memory
    /// * `items_count` - an estimation of the maximum number of items to store.
    pub fn new(bitmap_size: usize, items_count: usize) -> Self {
        assert!(bitmap_size > 0 && items_count > 0);
        let bitmap_bits = (bitmap_size as u64) * 8u64;
        let k_num = Self::optimal_k_num(bitmap_bits, items_count);
        let bitmap = BitVec::from_elem(bitmap_bits as usize, false);
        let sips = [Self::sip_new(), Self::sip_new()];
        let config = BloomHasher::new(bitmap_bits, k_num, sips);
        let state = BloomState{bitmap};
        Bloom {state, config, phantom: PhantomData}
    }

    /// Create a new bloom filter structure.
    ///
    /// * `items_count` - an estimation of the maximum number of items to store.
    /// * `fp_p` - the wanted rate of false positives, in ]0.0, 1.0[
    pub fn new_for_fp_rate(items_count: usize, fp_p: f64) -> Self {
        let bitmap_size = Self::compute_bitmap_size(items_count, fp_p);
        Bloom::new(bitmap_size, items_count)
    }

    /// Create a bloom filter structure with an existing state.
    ///
    /// state is assumed to be retrieved from an existing bloom filter.
    pub fn from_existing(
        bitmap: &[u8],
        bitmap_bits: u64,
        k_num: u32,
        sip_keys: [(u64, u64); 2],
    ) -> Self {
        let sips = [
            SipHasher13::new_with_keys(sip_keys[0].0, sip_keys[0].1),
            SipHasher13::new_with_keys(sip_keys[1].0, sip_keys[1].1),
        ];
        let bitmap = BitVec::from_bytes(bitmap);
        Bloom {
            state: BloomState{bitmap},
            config: BloomHasher::new(bitmap_bits, k_num, sips),
            phantom: PhantomData,
        }
    }

    fn optimal_k_num(bitmap_bits: u64, items_count: usize) -> u32 {
        let m = bitmap_bits as f64;
        let n = items_count as f64;
        let k_num = (m / n * f64::ln(2.0f64)).ceil() as u32;
        cmp::max(k_num, 1)
    }

    fn sip_new() -> SipHasher13 {
        let mut rng = rand::thread_rng();
        SipHasher13::new_with_keys(
            rand::Rand::rand(&mut rng),
            rand::Rand::rand(&mut rng)
        )
    }

    /// Get a reference to the configuration used to build this Bloom filter.
    ///
    /// This hasher can be used to calculate offsets to be used with this Bloom
    /// filter later. This is mostly useful the actual hashing operation is very
    /// expensive or is being done in another thread
    pub fn get_hasher<'a>(&'a self) -> &'a BloomHasher<T> {
        &self.config
    }

    /// Compute a recommended bitmap size for `items_count` items
    /// and a `fp_p` rate of false positives.
    ///
    /// `fp_p` obviously has to be within the ]0.0, 1.0[ range.
    pub fn compute_bitmap_size(items_count: usize, fp_p: f64) -> usize {
        assert!(items_count > 0);
        assert!(fp_p > 0.0 && fp_p < 1.0);
        let log2 = f64::consts::LN_2;
        let log2_2 = log2 * log2;
        ((items_count as f64) * f64::ln(fp_p) / (-8.0 * log2_2)).ceil() as usize
    }

    /// Record the presence of an item.
    pub fn set(&mut self, item: &T) {
        let offsets = self.make_offsets(item);
        self.set_offsets(&offsets);
    }

    /// Record the presence of an item (using pre-built offsets)
    pub fn set_offsets(&mut self, offsets: &[usize]) {
        for offset in offsets {
            self.state.bitmap.set(*offset, true);
        }
    }

    /// Check if an item is present in the set.
    ///
    /// There can be false positives, but no false negatives.
    pub fn check(&self, item: &T) -> bool {
        let offsets = self.make_offsets(item);
        self.check_offsets(&offsets)
    }

    /// Check if an item is present in the set (using pre-built offsets).
    ///
    /// There can be false positives, but no false negatives.
    pub fn check_offsets(&self, offsets: &[usize]) -> bool {
        for offset in offsets {
            if self.state.bitmap.get(*offset).unwrap() == false {
                return false
            }
        }
        true
    }

    /// Construct offsets for use later with `set_offsets`/`check_offsets`
    pub fn make_offsets(&self, item: &T) -> Vec<usize> {
        self.config.make_offsets(item)
    }

    /// Record the presence of an item in the set,
    /// and return the previous state of this item.
    pub fn check_and_set(&mut self, item: &T) -> bool {
        let offsets = self.make_offsets(item);
        if self.check_offsets(&offsets) {
            true
        } else {
            self.set_offsets(&offsets);
            false
        }
    }

    /// Return the bitmap as a vector of bytes
    pub fn bitmap(&self) -> Vec<u8> {
        self.state.bitmap.to_bytes()
    }

    /// Return the bitmap as a read-only reference to the internal BitVec
    pub fn bitmap_ref<'a>(&'a self) -> &'a BitVec {
        &self.state.bitmap
    }

    /// Return the number of bits in the filter
    pub fn number_of_bits(&self) -> u64 {
        self.config.bitmap_bits
    }

    /// Return the number of hash functions used for `check` and `set`
    pub fn number_of_hash_functions(&self) -> u32 {
        self.config.k_num
    }

    /// Return the keys used by the sip hasher
    pub fn sip_keys(&self) -> [(u64, u64); 2] {
        self.config.sip_keys()
    }

    /// Clear all of the bits in the filter, removing all keys from the set
    pub fn clear(&mut self) {
        self.state.bitmap.clear()
    }
}

impl<T: Hash> BloomHasher<T> {
    fn new(bitmap_bits: u64, k_num: u32, sips: [SipHasher13;2]) -> Self {
        BloomHasher{
            bitmap_bits, k_num, sips,
            phantom: PhantomData
        }
    }

    pub fn make_offsets(&self, item: &T) -> Vec<usize> {
        let mut hashes = [0u64, 0u64];
        let mut ret = Vec::with_capacity(self.k_num as usize);
        for k_i in 0..self.k_num {
            let hashed = self.bloom_hash(&mut hashes, item, k_i);
            let bit_offset = (hashed % self.bitmap_bits) as usize;
            ret.push(bit_offset)
        }
        ret
    }

    fn bloom_hash(&self, hashes: &mut [u64; 2], item: &T, k_i: u32) -> u64 {
        if k_i < 2 {
            let sip = &mut self.sips[k_i as usize].clone();
            item.hash(sip);
            let hash = sip.finish();
            hashes[k_i as usize] = hash;
            hash
        } else {
            hashes[0].wrapping_add((k_i as u64).wrapping_mul(hashes[1]) % 0xffffffffffffffc5)
        }
    }

    fn sip_keys(&self) -> [(u64, u64); 2] {
        [self.sips[0].keys(), self.sips[1].keys()]
    }
}

impl<T: Hash> Clone for BloomHasher<T> {
    fn clone(&self) -> Self {
        BloomHasher::new(self.bitmap_bits, self.k_num, self.sips.clone())
        // bitmap_bits: u64, k_num: u32, sips: [SipHasher13;2]
    }
}

#[test]
fn bloom_test_set() {
    let mut bloom = Bloom::new(10, 80);
    let key: &Vec<u8> = &rand::thread_rng().gen_iter::<u8>().take(16).collect();
    assert!(bloom.check(key) == false);
    bloom.set(&key);
    assert!(bloom.check(&key) == true);
}

#[test]
fn bloom_test_check_and_set() {
    let mut bloom = Bloom::new(10, 80);
    let key: Vec<u8> = rand::thread_rng().gen_iter::<u8>().take(16).collect();
    assert!(bloom.check_and_set(&key) == false);
    assert!(bloom.check_and_set(&key) == true);
}

#[test]
fn bloom_test_clear() {
    let mut bloom = Bloom::new(10, 80);
    let key: Vec<u8> = rand::thread_rng().gen_iter::<u8>().take(16).collect();
    bloom.set(&key);
    assert!(bloom.check(&key) == true);
    bloom.clear();
    assert!(bloom.check(&key) == false);
}

#[test]
fn bloom_test_load() {
    let mut original = Bloom::new(10, 80);
    let key: Vec<u8> = rand::thread_rng().gen_iter::<u8>().take(16).collect();
    original.set(&key);
    assert!(original.check(&key) == true);

    let cloned = Bloom::from_existing(&original.bitmap(),
                                      original.number_of_bits(),
                                      original.number_of_hash_functions(),
                                      original.sip_keys());
    assert!(cloned.check(&key) == true);
}

#[test]
fn bloom_hasher_check() {
    let mut original = Bloom::new(10, 80);
    let key: Vec<u8> = rand::thread_rng().gen_iter::<u8>().take(16).collect();
    let hasher = original.get_hasher().clone();
    let offsets = hasher.make_offsets(&key);

    assert!(original.check_offsets(&offsets) == false);
    original.set(&key);
    assert!(original.check_offsets(&offsets) == true);
}

#[test]
fn bloom_hasher_set() {
    let mut original = Bloom::new(10, 80);
    let key: Vec<u8> = rand::thread_rng().gen_iter::<u8>().take(16).collect();
    let hasher = original.get_hasher().clone();
    let offsets = hasher.make_offsets(&key);

    assert!(original.check(&key) == false);
    original.set_offsets(&offsets);
    assert!(original.check(&key) == true);
}
