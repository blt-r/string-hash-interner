#![no_std]
#![warn(unsafe_op_in_unsafe_fn, clippy::redundant_closure_for_method_calls)]

//! Caches strings efficiently, with minimal memory footprint and associates them with unique symbols.
//! These symbols allow constant time comparisons and look-ups to the underlying interned strings.
//!
//! ### Example: Interning & Symbols
//!
//! ```
//! // An interner with default symbol type and hasher
//! use string_hash_interner::DefaultStringInterner;
//!
//! let mut interner = DefaultStringInterner::default();
//! let sym0 = interner.intern("Elephant");
//! let sym1 = interner.intern("Tiger");
//! let sym2 = interner.intern("Horse");
//! let sym3 = interner.intern("Tiger");
//! assert_ne!(sym0, sym1);
//! assert_ne!(sym0, sym2);
//! assert_ne!(sym1, sym2);
//! assert_eq!(sym1, sym3); // same!
//! ```
//!
//! ### Example: Creation by `FromIterator`
//!
//! ```
//! # use string_hash_interner::DefaultStringInterner;
//! let interner = ["Elephant", "Tiger", "Horse", "Tiger"]
//!     .into_iter()
//!     .collect::<DefaultStringInterner>();
//! ```
//!
//! ### Example: Look-up
//!
//! ```
//! # use string_hash_interner::DefaultStringInterner;
//! let mut interner = DefaultStringInterner::default();
//! let sym = interner.intern("Banana");
//! assert_eq!(interner.resolve(sym), Some("Banana"));
//! ```
//!
//! ### Example: Iteration
//!
//! ```
//! # use string_hash_interner::{DefaultStringInterner, Symbol};
//! let interner = DefaultStringInterner::from_iter(["Earth", "Water", "Fire", "Air"]);
//! for (sym, str) in &interner {
//!     println!("{} = {}", sym.to_usize(), str);
//! }
//! ```
//!
//! ### Example: Use different symbols and hashers
//!
//! ```
//! # use string_hash_interner::StringInterner;
//! use string_hash_interner::symbol::SymbolU16;
//! use fxhash::FxBuildHasher;
//! let mut interner = StringInterner::<SymbolU16, FxBuildHasher>::new();
//! let sym = interner.intern("Fire Fox");
//! assert_eq!(interner.resolve(sym), Some("Fire Fox"));
//! assert_eq!(size_of_val(&sym), 2);
//! ```
//!
//! ### Example: Intern different types of strings
//!
//! ```
//! use string_hash_interner::Interner;
//! use std::ffi::CStr;
//!
//! let strings = <Interner<CStr>>::from_iter([c"Earth", c"Water", c"Fire", c"Air"]);
//!
//! for (_sym, str) in &strings {
//!     println!("This is a C string: {:?}", str);
//! }
//! ```
//!
//! ### Example: Use cached hashes for faster hashmap lookups
//!
//! ```
//! # use string_hash_interner::DefaultStringInterner;
//! # use string_hash_interner::DefaultHashBuilder;
//! # use hashbrown::hash_map::RawEntryMut;
//! // `DefaultHashBuilder` uses random state, so we need to use
//! // the same instance in order for hashes to match.
//! let build_hasher = DefaultHashBuilder::default();
//!
//! let mut hashmap = hashbrown::HashMap::with_hasher(build_hasher);
//! hashmap.extend([("Earth", 1), ("Water", 2), ("Fire", 3), ("Air", 4)]);
//!
//! let mut interner = DefaultStringInterner::with_hasher(build_hasher);
//! let sym = interner.intern("Water");
//!
//! // Now, if we need to lookup the entry in the hashmap and we
//! // only have the symbol, we don't need to recompute the hash.
//!
//! let string = interner.resolve(sym).unwrap();
//! let hash = interner.get_hash(sym).unwrap();
//!
//! let (k, v) = hashmap
//!     .raw_entry()
//!     .from_key_hashed_nocheck(hash, string)
//!     .unwrap();
//!
//! assert_eq!(*k, "Water");
//! assert_eq!(*v, 2)
//! ```
//!
//! ### Example: Hashmap with only interned strings
//!
//! ```
//! # use string_hash_interner::symbol::DefaultSymbol;
//! # use string_hash_interner::DefaultStringInterner;
//! # use hashbrown::hash_map::RawEntryMut;
//! let mut interner = DefaultStringInterner::default();
//!
//! let symbols = ["Earth", "Water", "Fire", "Air", "Air", "Water"].map(|s| interner.intern(s));
//!
//! // Now, using symbols we can fill the hashmap without ever recomputing hashes.
//!
//! // Use `()` as a hasher, as we'll be using cached hashes.
//! let mut counts = hashbrown::HashMap::<DefaultSymbol, usize, ()>::default();
//!
//! for symbol in symbols {
//!     // SAFETY: we now these symbols are coming from this interner
//!     let hash = unsafe { interner.get_hash_unchecked(symbol) };
//!     let hasher = |sym: &DefaultSymbol| unsafe { interner.get_hash_unchecked(*sym) };
//!
//!     match counts.raw_entry_mut().from_key_hashed_nocheck(hash, &symbol) {
//!         RawEntryMut::Occupied(mut entry) => {
//!             *entry.get_mut() += 1;
//!         }
//!         RawEntryMut::Vacant(entry) => {
//!             entry.insert_with_hasher(hash, symbol, 1, hasher);
//!         }
//!     }
//! }
//!
//! for (sym, count) in &counts {
//!     println!("{:?} appeared {} times", interner.resolve(*sym).unwrap(), count);
//! }
//! ```
//!

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "serde")]
mod serde_impl;

mod backend;
mod intern;
mod interner;
pub mod symbol;

#[doc(inline)]
pub use self::{
    backend::{Iter, IterWithHashes},
    intern::Intern,
    interner::Interner,
    symbol::{DefaultSymbol, Symbol},
};

#[doc(inline)]
pub use hashbrown::DefaultHashBuilder;

/// [`Interner`] for [`str`]'s.
pub type StringInterner<S = DefaultSymbol, H = DefaultHashBuilder> = Interner<str, S, H>;

/// [`StringInterner`] with default Symbol and Hasher.
pub type DefaultStringInterner = StringInterner;
