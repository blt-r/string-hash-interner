use std::hash::{BuildHasher, Hash, Hasher};

use fxhash::{FxBuildHasher, FxHasher};
use string_interner::{
    DefaultHashBuilder, DefaultStringInterner as StringInterner, DefaultSymbol, Symbol,
};

fn expect_valid_symbol<S>(index: usize) -> S
where
    S: Symbol,
{
    S::try_from_usize(index).expect("encountered invalid symbol")
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
        .map(|&s| (interner.intern(s), s))
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
fn correct_hashes() {
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
fn correct_fxhashes() {
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

#[test]
fn manual_hashmap() {
    // Force at least one rehashing
    let strings = (0..512).map(|n| format!("string {n}")).collect::<Vec<_>>();

    let build_hasher = DefaultHashBuilder::default();

    let mut interner = StringInterner::with_hasher(build_hasher);

    let symbols = strings
        .iter()
        .map(|s| interner.intern(s))
        .collect::<Vec<_>>();

    let mut hashmap = hashbrown::HashMap::<Box<str>, usize, _>::with_hasher(build_hasher);

    for (i, sym) in symbols.iter().enumerate() {
        let hash = interner.get_hash(*sym).unwrap();
        let s = interner.resolve(*sym).unwrap();

        hashmap
            .raw_entry_mut()
            .from_key_hashed_nocheck(hash, s)
            .insert(s.into(), i);
    }

    for (i, sym) in symbols.iter().enumerate() {
        let hash = interner.get_hash(*sym).unwrap();
        let s = interner.resolve(*sym).unwrap();

        let (k, v) = match hashmap.raw_entry_mut().from_key_hashed_nocheck(hash, s) {
            hashbrown::hash_map::RawEntryMut::Occupied(entry) => entry.into_key_value(),
            hashbrown::hash_map::RawEntryMut::Vacant(_) => panic!(),
        };

        assert_eq!(k.as_ref(), s);
        assert_eq!(k.as_ref(), strings[*v]);
        assert_eq!(*v, i);
    }
}

#[test]
fn iter_with_hashes() {
    let strings = ["aa", "bb", "cc", "dd", "ee", "ff"];
    let build_hasher = DefaultHashBuilder::default();

    let mut interner = StringInterner::with_hasher(build_hasher);

    let make_hash = |s: &str| {
        let mut hasher = build_hasher.build_hasher();
        s.hash(&mut hasher);
        hasher.finish()
    };

    let expected = strings
        .iter()
        .map(|&s| (interner.intern(s), s, make_hash(s)))
        .collect::<Vec<_>>();

    assert!(Iterator::eq(interner.iter_with_hashes(), expected));
}

mod different_strings {
    use std::{
        borrow::Borrow,
        ffi::{CStr, CString, OsStr},
        hash::{BuildHasher, Hasher},
    };

    use hashbrown::DefaultHashBuilder;
    use string_interner::{Intern, Interner};

    trait TestString: Intern + ToOwned + AsRef<Self> {
        fn make(s: &str) -> Self::Owned;

        fn data(data: impl IntoIterator<Item = &'static str>) -> Vec<Self::Owned> {
            data.into_iter().map(Self::make).collect()
        }
    }

    impl TestString for str {
        fn make(s: &str) -> Self::Owned {
            s.to_owned()
        }
    }

    impl TestString for CStr {
        fn make(s: &str) -> Self::Owned {
            CString::new(s).unwrap()
        }
    }

    impl TestString for OsStr {
        fn make(s: &str) -> Self::Owned {
            From::from(s)
        }
    }

    impl TestString for [u8] {
        fn make(s: &str) -> Self::Owned {
            s.as_bytes().to_vec()
        }
    }

    impl TestString for [char] {
        fn make(s: &str) -> Self::Owned {
            s.chars().collect()
        }
    }

    fn general_test<I: TestString + ?Sized>() {
        let strings = I::data(["aa", "bb", "cc", "dd", "ee", "ff"]);

        let build_hasher = DefaultHashBuilder::default();
        let make_hash = |s: &I| {
            let mut hasher = build_hasher.build_hasher();
            s.hash(&mut hasher);
            hasher.finish()
        };

        let mut interner = Interner::<I>::with_hasher(build_hasher);

        let expected = strings
            .iter()
            .map(Borrow::borrow)
            .map(|s| (interner.intern(s), s, make_hash(s)))
            .collect::<Vec<_>>();

        assert!(Iterator::eq(interner.iter_with_hashes(), expected));
    }

    #[test]
    fn all_string_types() {
        general_test::<str>();
        general_test::<CStr>();
        general_test::<OsStr>();
        general_test::<[u8]>();
        general_test::<[char]>();
    }
}
