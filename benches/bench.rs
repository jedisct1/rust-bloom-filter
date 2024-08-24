#![feature(test)]

extern crate test;

use bloomfilter::Bloom;

/* Set benchmarks */

fn inner_insert_bench(b: &mut test::Bencher, bitmap_size: usize, items_count: usize) {
    let mut bf: Bloom<usize> = Bloom::new(bitmap_size / 8, items_count);
    let mut index = items_count;
    b.iter(|| {
        index += 1;
        test::black_box(bf.set(&index));
    });
}

#[bench]
#[inline(always)]
fn bench_insert_100(b: &mut test::Bencher) {
    inner_insert_bench(b, 1000, 100);
}


#[bench]
#[inline(always)]
fn bench_insert_1000(b: &mut test::Bencher) {
    inner_insert_bench(b, 10000, 1000);
}

#[bench]
#[inline(always)]
fn bench_insert_m_1(b: &mut test::Bencher) {
    inner_insert_bench(b, 10_000_000, 1_000_000);
}

#[bench]
#[inline(always)]
fn bench_insert_m_10(b: &mut test::Bencher) {
    inner_insert_bench(b, 100_000_000, 10_000_000);
}

/* Get benchmarks */

fn inner_get_bench(b: &mut test::Bencher, bitmap_size: usize, items_count: usize) {
    let mut bf: Bloom<usize> = Bloom::new(bitmap_size / 8, items_count);
    for index in 0..items_count {
        bf.set(&index);
    }
    let mut index = items_count;
    b.iter(|| {
        index += 1;
        test::black_box(bf.check(&index));
    });
}


#[bench]
#[inline(always)]
fn bench_get_100(b: &mut test::Bencher) {
    inner_get_bench(b, 1000, 100);
}


#[bench]
#[inline(always)]
fn bench_get_1000(b: &mut test::Bencher) {
    inner_get_bench(b, 10000, 1000);
}


#[bench]
#[inline(always)]
fn bench_get_m_1(b: &mut test::Bencher) {
    inner_get_bench(b, 10_000_000, 1_000_000);
}

#[bench]
#[inline(always)]
fn bench_get_m_10(b: &mut test::Bencher) {
    inner_get_bench(b, 100_000_000, 10_000_000);
}
