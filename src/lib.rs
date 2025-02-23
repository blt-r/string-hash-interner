#![no_std]
#![doc(html_root_url = "https://docs.rs/crate/string-interner/0.18.0")]
#![warn(unsafe_op_in_unsafe_fn, clippy::redundant_closure_for_method_calls)]

//! Caches strings efficiently, with minimal memory footprint and associates them with unique symbols.
//! These symbols allow constant time comparisons and look-ups to the underlying interned strings.
//!
//! ### Example: Interning & Symbols
//!
//! ```
//! // An interner with default symbol type and hasher
//! use string_interner::DefaultStringInterner;
//!
//! let mut interner = DefaultStringInterner::default();
//! let sym0 = interner.get_or_intern("Elephant");
//! let sym1 = interner.get_or_intern("Tiger");
//! let sym2 = interner.get_or_intern("Horse");
//! let sym3 = interner.get_or_intern("Tiger");
//! assert_ne!(sym0, sym1);
//! assert_ne!(sym0, sym2);
//! assert_ne!(sym1, sym2);
//! assert_eq!(sym1, sym3); // same!
//! ```
//!
//! ### Example: Creation by `FromIterator`
//!
//! ```
//! # use string_interner::DefaultStringInterner;
//! let interner = ["Elephant", "Tiger", "Horse", "Tiger"]
//!     .into_iter()
//!     .collect::<DefaultStringInterner>();
//! ```
//!
//! ### Example: Look-up
//!
//! ```
//! # use string_interner::DefaultStringInterner;
//! let mut interner = DefaultStringInterner::default();
//! let sym = interner.get_or_intern("Banana");
//! assert_eq!(interner.resolve(sym), Some("Banana"));
//! ```
//!
//! ### Example: Iteration
//!
//! ```
//! # use string_interner::{DefaultStringInterner, Symbol};
//! let interner = DefaultStringInterner::from_iter(["Earth", "Water", "Fire", "Air"]);
//! for (sym, str) in &interner {
//!     println!("{} = {}", sym.to_usize(), str);
//! }
//! ```
//!
//! ### Example: Use different symbols and hashers:
//!
//! ```
//! # use string_interner::StringInterner;
//! use string_interner::symbol::SymbolU16;
//! use fxhash::FxBuildHasher;
//! let mut interner = StringInterner::<SymbolU16, FxBuildHasher>::new();
//! let sym = interner.get_or_intern("Fire Fox");
//! assert_eq!(interner.resolve(sym), Some("Fire Fox"));
//! assert_eq!(size_of_val(&sym), 2);
//! ```
//!

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "serde")]
mod serde_impl;

mod backend;
mod interner;
pub mod symbol;

#[doc(inline)]
pub use self::{
    backend::Iter,
    interner::StringInterner,
    symbol::{DefaultSymbol, Symbol},
};

/// [StringInterner] with default Symbol and Hasher.
pub type DefaultStringInterner = StringInterner;

#[doc(inline)]
pub use hashbrown::DefaultHashBuilder;
