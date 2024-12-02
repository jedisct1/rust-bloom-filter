use bloomfilter::{reexports::getrandom::getrandom, Bloom};

#[test]
#[cfg(feature = "random")]
fn bloom_test_set() {
    let mut bloom = Bloom::new(10, 80).unwrap();
    let mut k = vec![0u8, 16];
    getrandom(&mut k).unwrap();
    assert!(!bloom.check(&k));
    bloom.set(&k);
    assert!(bloom.check(&k));
}

#[test]
#[cfg(feature = "random")]
fn bloom_test_check_and_set() {
    let mut bloom = Bloom::new(10, 80).unwrap();
    let mut k = vec![0u8, 16];
    getrandom(&mut k).unwrap();
    assert!(!bloom.check_and_set(&k));
    assert!(bloom.check_and_set(&k));
}

#[test]
#[cfg(feature = "random")]
fn bloom_test_clear() {
    let mut bloom = Bloom::new(10, 80).unwrap();
    let mut k = vec![0u8, 16];
    getrandom(&mut k).unwrap();
    bloom.set(&k);
    assert!(bloom.check(&k));
    bloom.clear();
    assert!(!bloom.check(&k));
}

#[test]
#[cfg(feature = "random")]
fn bloom_test_load() {
    let mut original = Bloom::new(10, 80).unwrap();
    let mut k = vec![0u8, 16];
    getrandom(&mut k).unwrap();
    original.set(&k);
    assert!(original.check(&k));

    let original_bytes = original.as_slice();
    let cloned = Bloom::from_slice(original_bytes).unwrap();
    let cloned_bytes = cloned.to_bytes();
    assert_eq!(original_bytes, cloned_bytes);
    assert!(original.check(&k));
    assert!(cloned.check(&k));
}
