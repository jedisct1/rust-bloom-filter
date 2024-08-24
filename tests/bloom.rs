use bloomfilter::{reexports::getrandom::getrandom, Bloom};

#[test]
#[cfg(feature = "random")]
fn bloom_test_set() {
    let mut bloom = Bloom::new(10, 80);
    let mut k = vec![0u8, 16];
    getrandom(&mut k).unwrap();
    assert!(!bloom.check(&k));
    bloom.set(&k);
    assert!(bloom.check(&k));
}

#[test]
#[cfg(feature = "random")]
fn bloom_test_check_and_set() {
    let mut bloom = Bloom::new(10, 80);
    let mut k = vec![0u8, 16];
    getrandom(&mut k).unwrap();
    assert!(!bloom.check_and_set(&k));
    assert!(bloom.check_and_set(&k));
}

#[test]
#[cfg(feature = "random")]
fn bloom_test_clear() {
    let mut bloom = Bloom::new(10, 80);
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
    let mut original = Bloom::new(10, 80);
    let mut k = vec![0u8, 16];
    getrandom(&mut k).unwrap();
    original.set(&k);
    assert!(original.check(&k));

    let cloned = Bloom::from_existing(
        &original.bitmap(),
        original.number_of_bits(),
        original.number_of_hash_functions(),
        original.sip_keys(),
    );
    assert!(cloned.check(&k));
}

/// Test the false positive rate of the bloom filter
/// to ensure that using floor doesn't affect false positive rate
/// in a significant way
#[test]
fn test_false_positive_rate() {
    let capacities = [100, 1000, 10000, 100000, 1000000];
    for capacity in capacities.iter() {
        let mut bf: Bloom<usize> = Bloom::new(*capacity * 10 / 8, *capacity);
        for index in 0..*capacity {
            bf.set(&index);
        }
        let mut false_positives_count = 0.0;
        for index in *capacity..11 * *capacity {
            if bf.check(&index) {
                false_positives_count += 1.0;
            }
        }
        println!("False positive rate for capacity {}: {}", *capacity, false_positives_count / (10.0 * *capacity as f64));
    }
}