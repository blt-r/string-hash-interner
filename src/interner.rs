use crate::{
    backend::{Iter, IterWithHashes, StringBackend},
    intern::Intern,
    DefaultSymbol, Symbol,
};
use core::{
    fmt,
    fmt::{Debug, Formatter},
    hash::{BuildHasher, Hasher},
    iter::FromIterator,
};
use hashbrown::{DefaultHashBuilder, HashMap};

/// Creates the `u64` hash value for the given value using the given hash builder.
fn make_hash<I: Intern + ?Sized>(builder: &impl BuildHasher, value: &I) -> u64 {
    let state = &mut builder.build_hasher();
    value.hash(state);
    state.finish()
}

/// Data structure to intern and resolve strings.
///
/// Caches strings efficiently, with minimal memory footprint and associates them with unique symbols.
/// These symbols allow constant time comparisons and look-ups to the underlying interned strings.
///
/// The following API covers the main functionality:
///
/// - [`Interner::intern`]: To intern a new string.
///     - This maps from `string` type to `symbol` type.
/// - [`Interner::resolve`]: To resolve your already interned strings.
///     - This maps from `symbol` type to `string` type.
pub struct Interner<I: Intern + ?Sized, S: Symbol = DefaultSymbol, H = DefaultHashBuilder> {
    dedup: HashMap<S, (), ()>,
    hasher: H,
    backend: StringBackend<I, S>,
}

impl<I: Intern + ?Sized, S: Symbol, H> Debug for Interner<I, S, H>
where
    S: Debug,
    H: BuildHasher,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("StringInterner")
            .field("dedup", &self.dedup)
            .field("backend", &self.backend)
            .finish()
    }
}

impl<I: Intern + ?Sized, S: Symbol, H: BuildHasher + Default> Default for Interner<I, S, H> {
    #[cfg_attr(feature = "inline-more", inline)]
    fn default() -> Self {
        Interner::new()
    }
}

impl<I: Intern + ?Sized, S: Symbol, H: Clone> Clone for Interner<I, S, H> {
    fn clone(&self) -> Self {
        Self {
            dedup: self.dedup.clone(),
            hasher: self.hasher.clone(),
            backend: self.backend.clone(),
        }
    }
}

impl<I: Intern + ?Sized, S: Symbol, H: BuildHasher + Default> Interner<I, S, H> {
    /// Creates a new empty [Interner].
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn new() -> Self {
        Self {
            dedup: HashMap::default(),
            hasher: Default::default(),
            backend: StringBackend::default(),
        }
    }

    /// Creates a new `StringInterner` with the given initial capacity.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            dedup: HashMap::with_capacity_and_hasher(cap, ()),
            hasher: Default::default(),
            backend: StringBackend::with_capacity(cap),
        }
    }
}

impl<I: Intern + ?Sized, S: Symbol, H: BuildHasher> Interner<I, S, H> {
    /// Creates a new empty `StringInterner` with the given hasher.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn with_hasher(hash_builder: H) -> Self {
        Interner {
            dedup: HashMap::default(),
            hasher: hash_builder,
            backend: StringBackend::default(),
        }
    }

    /// Creates a new empty `StringInterner` with the given initial capacity and the given hasher.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn with_capacity_and_hasher(cap: usize, hash_builder: H) -> Self {
        Interner {
            dedup: HashMap::with_capacity_and_hasher(cap, ()),
            hasher: hash_builder,
            backend: StringBackend::with_capacity(cap),
        }
    }

    /// Returns the number of strings interned by the interner.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn len(&self) -> usize {
        self.dedup.len()
    }

    /// Returns `true` if the string interner has no interned strings.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the symbol for the given string if any.
    ///
    /// Can be used to query if a string has already been interned without interning.
    #[inline]
    pub fn get<T>(&self, string: T) -> Option<S>
    where
        T: AsRef<I>,
    {
        let string = string.as_ref();

        let hash = make_hash(&self.hasher, string);
        self.dedup
            .raw_entry()
            .from_hash(hash, |symbol| {
                // SAFETY: This is safe because we only operate on symbols that
                //         we receive from our backend making them valid.
                string == unsafe { self.backend.resolve_unchecked(*symbol) }
            })
            .map(|(&symbol, &())| symbol)
    }

    /// Interns the given string.
    ///
    /// Returns a symbol for resolution into the original string, and its hash.
    ///
    /// # Panics
    ///
    /// If the interner already interns the maximum number of strings possible
    /// by the chosen symbol type.
    #[inline]
    pub fn intern_and_hash<T: AsRef<I>>(&mut self, string: T) -> (S, u64) {
        let string = string.as_ref();

        let hash = make_hash(&self.hasher, string);
        let entry = self.dedup.raw_entry_mut().from_hash(hash, |symbol| {
            // SAFETY: This is safe because we only operate on symbols that
            //         we receive from our backend making them valid.
            string == unsafe { self.backend.resolve_unchecked(*symbol) }
        });
        use hashbrown::hash_map::RawEntryMut;
        let (&mut symbol, &mut ()) = match entry {
            RawEntryMut::Occupied(occupied) => occupied.into_key_value(),
            RawEntryMut::Vacant(vacant) => {
                let symbol = self.backend.intern(string, hash);
                vacant.insert_with_hasher(hash, symbol, (), |symbol| {
                    // SAFETY: This is safe because we only operate on symbols that
                    //         we receive from our backend making them valid.
                    unsafe { self.backend.get_hash_unchecked(*symbol) }
                })
            }
        };
        (symbol, hash)
    }

    /// Interns the given string.
    ///
    /// Returns a symbol for resolution into the original string.
    ///
    /// # Panics
    ///
    /// If the interner already interns the maximum number of strings possible
    /// by the chosen symbol type.
    #[inline]
    pub fn intern<T: AsRef<I>>(&mut self, string: T) -> S {
        self.intern_and_hash(string).0
    }

    /// Shrink backend capacity to fit the interned strings exactly.
    pub fn shrink_to_fit(&mut self) {
        self.backend.shrink_to_fit()
    }

    /// Returns the string for the given `symbol`` if any.
    #[inline]
    pub fn resolve(&self, symbol: S) -> Option<&I> {
        self.backend.resolve(symbol)
    }

    /// Returns cached hash of the string for the given `symbol`.
    pub fn get_hash(&self, symbol: S) -> Option<u64> {
        self.backend.get_hash(symbol)
    }

    /// Returns the string for the given `symbol` without performing any checks.
    ///
    /// # Safety
    ///
    /// It is the caller's responsibility to provide this method with `symbol`s
    /// that are valid for the [Interner].
    #[inline]
    pub unsafe fn resolve_unchecked(&self, symbol: S) -> &I {
        unsafe { self.backend.resolve_unchecked(symbol) }
    }

    /// Returns cached hash of the string for the given `symbol` without performing any checks.
    ///
    /// # Safety
    ///
    /// It is the caller's responsibility to provide this method with `symbol`s
    /// that are valid for the [Interner].
    pub unsafe fn get_hash_unchecked(&self, symbol: S) -> u64 {
        // SAFETY: The function is marked unsafe so that the caller guarantees
        //         that required invariants are checked.
        unsafe { self.backend.get_hash_unchecked(symbol) }
    }

    /// Returns an iterator that yields all interned strings, their symbols, and hashes.
    #[inline]
    pub fn iter_with_hashes(&self) -> IterWithHashes<'_, I, S> {
        self.backend.iter_with_hashes()
    }

    /// Returns an iterator that yields all interned strings and their symbols.
    #[inline]
    pub fn iter(&self) -> Iter<'_, I, S> {
        self.backend.iter()
    }
}

impl<I: Intern + ?Sized, S: Symbol, H: BuildHasher + Default, T: AsRef<I>> FromIterator<T>
    for Interner<I, S, H>
{
    fn from_iter<It>(iter: It) -> Self
    where
        It: IntoIterator<Item = T>,
    {
        let iter = iter.into_iter();
        let (capacity, _) = iter.size_hint();
        let mut interner = Self::with_capacity(capacity);
        interner.extend(iter);
        interner
    }
}

impl<I: Intern + ?Sized, S: Symbol, H: BuildHasher + Default, T: AsRef<I>> Extend<T>
    for Interner<I, S, H>
{
    fn extend<It>(&mut self, iter: It)
    where
        It: IntoIterator<Item = T>,
    {
        for s in iter {
            self.intern_and_hash(s);
        }
    }
}

impl<'a, I: Intern + ?Sized, S: Symbol, H> IntoIterator for &'a Interner<I, S, H> {
    type Item = (S, &'a I);
    type IntoIter = Iter<'a, I, S>;

    #[cfg_attr(feature = "inline-more", inline)]
    fn into_iter(self) -> Self::IntoIter {
        self.backend.iter()
    }
}
