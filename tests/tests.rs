mod allocator;

use std::hash::{BuildHasher, Hash, Hasher};

use allocator::TracingAllocator;
use fxhash::{FxBuildHasher, FxHasher};
use string_interner::{DefaultHashBuilder, DefaultSymbol, Symbol};

#[global_allocator]
static ALLOCATOR: TracingAllocator = TracingAllocator::new();

/// Creates the symbol `S` from the given `usize`.
///
/// # Panics
///
/// Panics if the conversion is invalid.
#[inline]
pub(crate) fn expect_valid_symbol<S>(index: usize) -> S
where
    S: Symbol,
{
    S::try_from_usize(index).expect("encountered invalid symbol")
}

/// Stats for the backend.
mod stats {
    pub const MIN_OVERHEAD: f64 = 1.7;
    pub const MAX_OVERHEAD: f64 = 1.93;
    pub const MAX_ALLOCATIONS: usize = 62;
    pub const MAX_DEALLOCATIONS: usize = 59;
    pub const NAME: &'static str = "StringBackend";
}

/// Memory profiling stats.
pub struct ProfilingStats {
    /// The minimum memory usage overhead as factor.
    pub overhead: f64,
    /// The total amount of allocations of the profiling test.
    pub allocations: usize,
    /// The total amount of deallocations of the profiling test.
    pub deallocations: usize,
}

type StringInterner = string_interner::StringInterner<DefaultSymbol, DefaultHashBuilder>;

fn profile_memory_usage(words: &[String]) -> ProfilingStats {
    ALLOCATOR.reset();
    ALLOCATOR.start_profiling();
    let mut interner = StringInterner::new();
    ALLOCATOR.end_profiling();

    for word in words {
        ALLOCATOR.start_profiling();
        interner.intern(word);
    }
    interner.shrink_to_fit();
    ALLOCATOR.end_profiling();

    let stats = ALLOCATOR.stats();
    let len_allocations = stats.len_allocations();
    let len_deallocations = stats.len_deallocations();
    let current_allocated_bytes = stats.current_allocated_bytes();
    let total_allocated_bytes = stats.total_allocated_bytes();

    assert_eq!(interner.len(), words.len());

    println!(
        "\
                \n\t- # words         = {}\
                \n\t- # allocations   = {}\
                \n\t- # deallocations = {}\
                \n\t- allocated bytes = {}\
                \n\t- requested bytes = {}\
                ",
        words.len(),
        len_allocations,
        len_deallocations,
        current_allocated_bytes,
        total_allocated_bytes,
    );

    let ideal_memory_usage = words.len() * words[0].len();
    let memory_usage_overhead = (current_allocated_bytes as f64) / (ideal_memory_usage as f64);
    println!("\t- ideal allocated bytes  = {}", ideal_memory_usage);
    println!("\t- actual allocated bytes = {}", current_allocated_bytes);
    println!(
        "\t- % actual overhead      = {:.02}%",
        memory_usage_overhead * 100.0
    );

    ProfilingStats {
        overhead: memory_usage_overhead,
        allocations: len_allocations,
        deallocations: len_deallocations,
    }
}

#[test]
#[cfg_attr(any(miri, not(feature = "test-allocations")), ignore)]
fn test_memory_consumption() {
    let len_words = 1_000_000;
    let words = (0..)
        .take(len_words)
        .map(|i| format!("{:20}", i))
        .collect::<Vec<_>>();

    println!();
    println!("Benchmark Memory Usage for {}", stats::NAME);
    let mut min_overhead = None;
    let mut max_overhead = None;
    let mut max_allocations = None;
    let mut max_deallocations = None;
    for i in 0..10 {
        let len_words = 100_000 * (i + 1);
        let words = &words[0..len_words];
        let stats = profile_memory_usage(words);
        if min_overhead.map(|min| stats.overhead < min).unwrap_or(true) {
            min_overhead = Some(stats.overhead);
        }
        if max_overhead.map(|max| stats.overhead > max).unwrap_or(true) {
            max_overhead = Some(stats.overhead);
        }
        if max_allocations
            .map(|max| stats.allocations > max)
            .unwrap_or(true)
        {
            max_allocations = Some(stats.allocations);
        }
        if max_deallocations
            .map(|max| stats.deallocations > max)
            .unwrap_or(true)
        {
            max_deallocations = Some(stats.deallocations);
        }
    }
    let actual_min_overhead = min_overhead.unwrap();
    let actual_max_overhead = max_overhead.unwrap();
    let expect_min_overhead = stats::MIN_OVERHEAD;
    let expect_max_overhead = stats::MAX_OVERHEAD;
    let actual_max_allocations = max_allocations.unwrap();
    let actual_max_deallocations = max_deallocations.unwrap();
    let expect_max_allocations = stats::MAX_ALLOCATIONS;
    let expect_max_deallocations = stats::MAX_DEALLOCATIONS;

    println!();
    println!(
        "- % min overhead      = {:.02}%",
        actual_min_overhead * 100.0
    );
    println!(
        "- % max overhead      = {:.02}%",
        actual_max_overhead * 100.0
    );
    println!("- % max allocations   = {}", actual_max_allocations);
    println!("- % max deallocations = {}", actual_max_deallocations);

    assert!(
                actual_min_overhead < expect_min_overhead,
                "{} string interner backend minimum memory overhead is greater than expected. expected = {:?}, actual = {:?}",
                stats::NAME,
                expect_min_overhead,
                actual_min_overhead,
            );
    assert!(
                actual_max_overhead < expect_max_overhead,
                "{} string interner backend maximum memory overhead is greater than expected. expected = {:?}, actual = {:?}",
                stats::NAME,
                expect_max_overhead,
                actual_max_overhead,
            );
    assert_eq!(
                actual_max_allocations, expect_max_allocations,
                "{} string interner backend maximum amount of allocations is greater than expected. expected = {:?}, actual = {:?}",
                stats::NAME,
                expect_max_allocations,
                actual_max_allocations,
            );
    assert_eq!(
                actual_max_deallocations, expect_max_deallocations,
                "{} string interner backend maximum amount of deallocations is greater than expected. expected = {:?}, actual = {:?}",
                stats::NAME,
                expect_max_deallocations,
                actual_max_deallocations,
            );
}

#[test]
fn new_works() {
    let interner = StringInterner::new();
    assert_eq!(interner.len(), 0);
    assert!(interner.is_empty());
    let other = StringInterner::new();
    assert!(Iterator::eq(interner.iter(), other.iter()));
}

#[test]
fn is_empty_works() {
    let mut interner = StringInterner::new();
    assert!(interner.is_empty());
    interner.intern("aa");
    assert!(!interner.is_empty());
}

#[test]
fn clone_works() {
    let mut interner = StringInterner::new();
    assert_eq!(interner.intern("aa").to_usize(), 0);

    let mut cloned = interner.clone();
    assert!(Iterator::eq(interner.iter(), cloned.iter()));
    // And the clone should have the same interned values
    assert_eq!(cloned.intern("aa").to_usize(), 0);
}

#[test]
fn intern_works() {
    let mut interner = StringInterner::new();
    // Insert 3 unique strings:
    let aa = interner.intern("aa").to_usize();
    let bb = interner.intern("bb").to_usize();
    let cc = interner.intern("cc").to_usize();
    // All symbols must be different from each other.
    assert_ne!(aa, bb);
    assert_ne!(bb, cc);
    assert_ne!(cc, aa);
    // The length of the string interner must be 3 at this point.
    assert_eq!(interner.len(), 3);
    // Insert the same 3 unique strings, yield the same symbols:
    assert_eq!(
        interner.resolve(<DefaultSymbol>::try_from_usize(aa).unwrap()),
        Some("aa")
    );
    assert_eq!(
        interner.intern("aa").to_usize(),
        aa,
        "'aa' did not produce the same symbol",
    );
    assert_eq!(
        interner.intern("bb").to_usize(),
        bb,
        "'bb' did not produce the same symbol",
    );
    assert_eq!(
        interner.intern("cc").to_usize(),
        cc,
        "'cc' did not produce the same symbol",
    );
    assert_eq!(interner.len(), 3);
}

#[test]
fn resolve_works() {
    let mut interner = StringInterner::new();
    // Insert 3 unique strings:
    let aa = interner.intern("aa");
    let bb = interner.intern("bb");
    let cc = interner.intern("cc");
    assert_eq!(interner.len(), 3);
    // Resolve valid symbols:
    assert_eq!(interner.resolve(aa), Some("aa"));
    assert_eq!(interner.resolve(bb), Some("bb"));
    assert_eq!(interner.resolve(cc), Some("cc"));
    assert_eq!(interner.len(), 3);
    // Resolve invalid symbols:
    let dd = expect_valid_symbol(1000);
    assert_ne!(aa, dd);
    assert_ne!(bb, dd);
    assert_ne!(cc, dd);
    assert_eq!(interner.resolve(dd), None);
}

#[test]
fn resolve_unchecked_works() {
    let mut interner = StringInterner::new();
    // Insert 3 unique strings:
    let aa = interner.intern("aa");
    let bb = interner.intern("bb");
    let cc = interner.intern("cc");
    assert_eq!(interner.len(), 3);
    // Resolve valid symbols:
    assert_eq!(unsafe { interner.resolve_unchecked(aa) }, "aa");
    assert_eq!(unsafe { interner.resolve_unchecked(bb) }, "bb");
    assert_eq!(unsafe { interner.resolve_unchecked(cc) }, "cc");
    assert_eq!(interner.len(), 3);
    // Resolve invalid symbols:
    let dd = expect_valid_symbol(1000);
    assert_ne!(aa, dd);
    assert_ne!(bb, dd);
    assert_ne!(cc, dd);
}

#[test]
fn get_works() {
    let mut interner = StringInterner::new();
    // Insert 3 unique strings:
    let aa = interner.intern("aa");
    let bb = interner.intern("bb");
    let cc = interner.intern("cc");
    assert_eq!(interner.len(), 3);
    // Get the symbols of the same 3 strings:
    assert_eq!(interner.get("aa"), Some(aa));
    assert_eq!(interner.get("bb"), Some(bb));
    assert_eq!(interner.get("cc"), Some(cc));
    assert_eq!(interner.len(), 3);
    // Get the symbols of some unknown strings:
    assert_eq!(interner.get("dd"), None);
    assert_eq!(interner.get("ee"), None);
    assert_eq!(interner.get("ff"), None);
    assert_eq!(interner.len(), 3);
}

#[test]
fn from_iter_works() {
    let strings = ["aa", "bb", "cc", "dd", "ee", "ff"];
    let expected = {
        let mut interner = StringInterner::new();
        for &string in &strings {
            interner.intern(string);
        }
        interner
    };
    let actual = strings.into_iter().collect::<StringInterner>();
    assert_eq!(actual.len(), strings.len());

    println!("{:?}", actual.iter().collect::<Vec<_>>());
    println!("{:?}", expected.iter().collect::<Vec<_>>());
    assert!(Iterator::eq(actual.iter(), expected.iter()));
}

#[test]
fn extend_works() {
    let strings = ["aa", "bb", "cc", "dd", "ee", "ff"];
    let expected = {
        let mut interner = StringInterner::new();
        for &string in &strings {
            interner.intern(string);
        }
        interner
    };
    let actual = {
        let mut interner = StringInterner::new();
        interner.extend(strings.iter().copied());
        interner
    };
    assert_eq!(actual.len(), strings.len());
    assert!(Iterator::eq(actual.iter(), expected.iter()));
}

#[test]
fn iter_works() {
    let mut interner = StringInterner::new();
    let strings = ["aa", "bb", "cc", "dd", "ee", "ff"];

    let symbols = strings
        .iter()
        .map(|&s| {
            let sym = interner.intern(s);
            (sym, s)
        })
        .collect::<Vec<_>>();

    assert!(Iterator::eq(symbols.into_iter(), &interner));
}

#[test]
fn shrink_to_fit_works() {
    let mut interner = StringInterner::new();
    // Insert 3 unique strings:
    let aa = interner.intern("aa").to_usize();
    let bb = interner.intern("bb").to_usize();
    let cc = interner.intern("cc").to_usize();

    interner.shrink_to_fit();

    assert_eq!(
        interner.intern("aa").to_usize(),
        aa,
        "'aa' did not produce the same symbol",
    );
    assert_eq!(
        interner.intern("bb").to_usize(),
        bb,
        "'bb' did not produce the same symbol",
    );
    assert_eq!(
        interner.intern("cc").to_usize(),
        cc,
        "'cc' did not produce the same symbol",
    );
    assert_eq!(interner.len(), 3);
}

#[test]
fn hashes_are_correct() {
    fn make_hash(build_hasher: impl BuildHasher, s: &str) -> u64 {
        let mut hasher = build_hasher.build_hasher();
        s.hash(&mut hasher);
        hasher.finish()
    }

    let hash_builder = DefaultHashBuilder::default();
    let mut interner = StringInterner::with_hasher(hash_builder);

    for s in ["aa", "bb", "cc", "dd", "ee", "ff"].iter().copied() {
        let expected_hash = make_hash(hash_builder, s);
        let (sym, hash) = interner.intern_and_hash(s);
        assert_eq!(expected_hash, hash);
        assert_eq!(expected_hash, interner.get_hash(sym).unwrap());
        assert_eq!(expected_hash, unsafe { interner.get_hash_unchecked(sym) });
    }
}

// FxHash isn't randomly seeded, so even with different [BuildHasher]'s the hashes should be the same.
#[test]
fn fxhashes_are_correct() {
    fn make_fxhash(s: &str) -> u64 {
        let mut hasher = FxHasher::default();
        s.hash(&mut hasher);
        hasher.finish()
    }

    let mut interner: string_interner::StringInterner<DefaultSymbol, FxBuildHasher> =
        Default::default();

    for s in ["aa", "bb", "cc", "dd", "ee", "ff"].iter().copied() {
        let expected_hash = make_fxhash(s);
        let (sym, hash) = interner.intern_and_hash(s);
        assert_eq!(expected_hash, hash);
        assert_eq!(expected_hash, interner.get_hash(sym).unwrap());
        assert_eq!(expected_hash, unsafe { interner.get_hash_unchecked(sym) });
    }
}
