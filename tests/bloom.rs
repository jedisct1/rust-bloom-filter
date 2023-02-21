use bloomfilter::{reexports::getrandom::getrandom, Bloom};

#[test]
#[cfg(feature = "random")]
fn bloom_test_set() {
    let mut bloom = Bloom::new(10, 80);
    let mut k = vec![0u8, 16];
    getrandom(&mut k).unwrap();
    assert!(bloom.check(&k) == false);
    bloom.set(&k);
    assert!(bloom.check(&k) == true);
}

#[test]
#[cfg(feature = "random")]
fn bloom_test_check_and_set() {
    let mut bloom = Bloom::new(10, 80);
    let mut k = vec![0u8, 16];
    getrandom(&mut k).unwrap();
    assert!(bloom.check_and_set(&k) == false);
    assert!(bloom.check_and_set(&k) == true);
}

#[test]
#[cfg(feature = "random")]
fn bloom_test_clear() {
    let mut bloom = Bloom::new(10, 80);
    let mut k = vec![0u8, 16];
    getrandom(&mut k).unwrap();
    bloom.set(&k);
    assert!(bloom.check(&k) == true);
    bloom.clear();
    assert!(bloom.check(&k) == false);
}

#[test]
#[cfg(feature = "random")]
fn bloom_test_load() {
    let mut original = Bloom::new(10, 80);
    let mut k = vec![0u8, 16];
    getrandom(&mut k).unwrap();
    original.set(&k);
    assert!(original.check(&k) == true);

    let cloned = Bloom::from_existing(
        &original.bitmap(),
        original.number_of_bits(),
        original.number_of_hash_functions(),
        original.sip_keys(),
    );
    assert!(cloned.check(&k) == true);
}
