use crate::{symbol::expect_valid_symbol, DefaultSymbol, Symbol};
use alloc::{string::String, vec::Vec};
use core::{iter::Enumerate, marker::PhantomData, slice};

/// An interner backend that accumulates all interned string contents into one string.
///
/// # Note
///
/// Implementation inspired by [CAD97's](https://github.com/CAD97) research
/// project [`strena`](https://github.com/CAD97/strena).
///
#[derive(Debug)]
pub(crate) struct StringBackend<S = DefaultSymbol> {
    /// Stores end of the string and it's hash
    ends: Vec<(usize, u64)>,
    buffer: String,
    marker: PhantomData<fn() -> S>,
}

/// Represents a `[from, to)` index into the `StringBackend` buffer.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Span {
    from: usize,
    to: usize,
}

impl<S> Clone for StringBackend<S> {
    fn clone(&self) -> Self {
        Self {
            ends: self.ends.clone(),
            buffer: self.buffer.clone(),
            marker: Default::default(),
        }
    }
}

impl<S> Default for StringBackend<S> {
    #[cfg_attr(feature = "inline-more", inline)]
    fn default() -> Self {
        Self {
            ends: Vec::default(),
            buffer: String::default(),
            marker: Default::default(),
        }
    }
}

impl<S> StringBackend<S>
where
    S: Symbol,
{
    /// Returns the next available symbol.
    fn next_symbol(&self) -> S {
        expect_valid_symbol(self.ends.len())
    }

    /// Returns the string associated to the span.
    ///
    /// # Safety
    ///
    /// Span must be valid withing the [Self::buffer]
    unsafe fn span_to_str(&self, span: Span) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.buffer.as_bytes()[span.from..span.to]) }
    }

    #[cfg_attr(feature = "inline-more", inline)]
    pub(crate) fn with_capacity(cap: usize) -> Self {
        // According to google the approx. word length is 5.
        let default_word_len = 5;
        Self {
            ends: Vec::with_capacity(cap),
            buffer: String::with_capacity(cap * default_word_len),
            marker: Default::default(),
        }
    }

    #[inline]
    pub(crate) fn intern(&mut self, string: &str, hash: u64) -> S {
        self.buffer.push_str(string);
        let to = self.buffer.len();
        let symbol = self.next_symbol();
        self.ends.push((to, hash));
        symbol
    }

    #[inline]
    pub(crate) fn resolve(&self, symbol: S) -> Option<&str> {
        let index = symbol.to_usize();
        let to = self.ends.get(index)?.0;

        let from = self
            .ends
            .get(index.wrapping_sub(1))
            .map(|&(end, _)| end)
            .unwrap_or(0);

        // SAFETY: This span is guaranteed to be valid
        unsafe { Some(self.span_to_str(Span { from, to })) }
    }

    pub(crate) fn shrink_to_fit(&mut self) {
        self.ends.shrink_to_fit();
        self.buffer.shrink_to_fit();
    }

    #[inline]
    pub(crate) unsafe fn resolve_unchecked(&self, symbol: S) -> &str {
        let index = symbol.to_usize();
        // SAFETY: The function is marked unsafe so that the caller guarantees
        //         that required invariants are checked.
        let to = unsafe { self.ends.get_unchecked(index).0 };
        let from = self
            .ends
            .get(index.wrapping_sub(1))
            .map(|&(end, _)| end)
            .unwrap_or(0);

        // SAFETY: This span is guaranteed to be valid
        unsafe { self.span_to_str(Span { from, to }) }
    }

    pub fn get_hash(&self, symbol: S) -> Option<u64> {
        self.ends.get(symbol.to_usize()).map(|&(_, hash)| hash)
    }

    pub unsafe fn get_hash_unchecked(&self, symbol: S) -> u64 {
        // SAFETY: The function is marked unsafe so that the caller guarantees
        //         that required invariants are checked.
        unsafe { self.ends.get_unchecked(symbol.to_usize()).1 }
    }

    #[inline]
    pub(crate) fn iter(&self) -> Iter<'_, S> {
        Iter::new(self)
    }

    #[inline]
    pub(crate) fn iter_with_hashes(&self) -> IterWithHashes<'_, S> {
        IterWithHashes::new(self)
    }
}

impl<'a, S> IntoIterator for &'a StringBackend<S>
where
    S: Symbol,
{
    type Item = (S, &'a str);
    type IntoIter = Iter<'a, S>;

    #[cfg_attr(feature = "inline-more", inline)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// An iterator over the interned symbols, their strings, and their hashes.
pub struct IterWithHashes<'a, S> {
    backend: &'a StringBackend<S>,
    start: usize,
    ends: Enumerate<slice::Iter<'a, (usize, u64)>>,
}

impl<'a, S> IterWithHashes<'a, S> {
    #[cfg_attr(feature = "inline-more", inline)]
    fn new(backend: &'a StringBackend<S>) -> Self {
        Self {
            backend,
            start: 0,
            ends: backend.ends.iter().enumerate(),
        }
    }
}

impl<'a, S> Iterator for IterWithHashes<'a, S>
where
    S: Symbol,
{
    type Item = (S, &'a str, u64);

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.ends.size_hint()
    }

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let (id, &(to, hash)) = self.ends.next()?;
        let from = core::mem::replace(&mut self.start, to);

        // SAFETY: This span is guaranteed to be valid
        let string = unsafe { self.backend.span_to_str(Span { from, to }) };

        Some((expect_valid_symbol(id), string, hash))
    }
}

/// An iterator over the interned symbols and their strings
pub struct Iter<'a, S> {
    inner: IterWithHashes<'a, S>,
}

impl<'a, S> Iter<'a, S> {
    #[cfg_attr(feature = "inline-more", inline)]
    fn new(backend: &'a StringBackend<S>) -> Self {
        Self {
            inner: IterWithHashes::new(backend),
        }
    }
}

impl<'a, S> Iterator for Iter<'a, S>
where
    S: Symbol,
{
    type Item = (S, &'a str);

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let (sym, s, _hash) = self.inner.next()?;
        Some((sym, s))
    }
}
