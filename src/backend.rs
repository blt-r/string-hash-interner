use crate::{intern::Intern, symbol::expect_valid_symbol, Symbol};
use alloc::vec::Vec;
use core::{fmt::Debug, iter::Enumerate, marker::PhantomData, slice};

/// An interner backend that accumulates all interned string contents into one string.
///
/// # Note
///
/// Implementation inspired by [CAD97's](https://github.com/CAD97) research
/// project [`strena`](https://github.com/CAD97/strena).
///
pub(crate) struct StringBackend<I: Intern + ?Sized, S> {
    /// Stores end of the string and it's hash
    ends: Vec<(usize, u64)>,
    buffer: Vec<I::Primitive>,
    marker: PhantomData<fn() -> S>,
}

impl<I: Intern + ?Sized, S> Debug for StringBackend<I, S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("StringBackend")
            .field("ends", &self.ends)
            .field("buffer", &self.buffer)
            .finish()
    }
}

impl<I: Intern + ?Sized, S> Clone for StringBackend<I, S> {
    fn clone(&self) -> Self {
        Self {
            ends: self.ends.clone(),
            buffer: self.buffer.clone(),
            marker: PhantomData,
        }
    }
}

impl<I: Intern + ?Sized, S> Default for StringBackend<I, S> {
    #[cfg_attr(feature = "inline-more", inline)]
    fn default() -> Self {
        Self {
            ends: Vec::default(),
            buffer: Vec::default(),
            marker: PhantomData,
        }
    }
}

impl<I: Intern + ?Sized, S: Symbol> StringBackend<I, S> {
    /// Returns the string associated to the span.
    ///
    /// # Safety
    ///
    /// Span must be valid within the [Self::buffer]
    unsafe fn span_to_str(&self, from: usize, to: usize) -> &I {
        unsafe { I::from_bytes(&self.buffer[from..to]) }
    }

    #[cfg_attr(feature = "inline-more", inline)]
    pub(crate) fn with_capacity(cap: usize) -> Self {
        // According to google the approx. word length is 5. So we will use 10.
        const DEFAULT_WORD_LEN: usize = 10;
        Self {
            ends: Vec::with_capacity(cap),
            buffer: Vec::with_capacity(cap * DEFAULT_WORD_LEN),
            marker: PhantomData,
        }
    }

    #[inline]
    pub(crate) fn intern(&mut self, string: &I, hash: u64) -> S {
        self.buffer.extend_from_slice(string.as_bytes());
        let to = self.buffer.len();
        let symbol = {
            let this = &self;
            expect_valid_symbol(this.ends.len())
        };
        self.ends.push((to, hash));
        symbol
    }

    #[inline]
    pub(crate) fn resolve(&self, symbol: S) -> Option<&I> {
        let index = symbol.to_usize();
        let to = self.ends.get(index)?.0;

        let from = self
            .ends
            .get(index.wrapping_sub(1))
            .map(|&(end, _)| end)
            .unwrap_or(0);

        // SAFETY: This span is guaranteed to be valid
        unsafe { Some(self.span_to_str(from, to)) }
    }

    pub(crate) fn shrink_to_fit(&mut self) {
        self.ends.shrink_to_fit();
        self.buffer.shrink_to_fit();
    }

    #[inline]
    pub(crate) unsafe fn resolve_unchecked(&self, symbol: S) -> &I {
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
        unsafe { self.span_to_str(from, to) }
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
    pub(crate) fn iter(&self) -> Iter<'_, I, S> {
        Iter::new(self)
    }

    #[inline]
    pub(crate) fn iter_with_hashes(&self) -> IterWithHashes<'_, I, S> {
        IterWithHashes::new(self)
    }
}

impl<'a, I: Intern + ?Sized, S: Symbol> IntoIterator for &'a StringBackend<I, S> {
    type Item = (S, &'a I);
    type IntoIter = Iter<'a, I, S>;

    #[cfg_attr(feature = "inline-more", inline)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// An iterator over the interned symbols, their strings, and their hashes.
pub struct IterWithHashes<'a, I: Intern + ?Sized, S> {
    backend: &'a StringBackend<I, S>,
    start: usize,
    ends: Enumerate<slice::Iter<'a, (usize, u64)>>,
}

impl<'a, I: Intern + ?Sized, S> IterWithHashes<'a, I, S> {
    #[cfg_attr(feature = "inline-more", inline)]
    fn new(backend: &'a StringBackend<I, S>) -> Self {
        Self {
            backend,
            start: 0,
            ends: backend.ends.iter().enumerate(),
        }
    }
}

impl<'a, I: Intern + ?Sized, S: Symbol> Iterator for IterWithHashes<'a, I, S> {
    type Item = (S, &'a I, u64);

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.ends.size_hint()
    }

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let (id, &(to, hash)) = self.ends.next()?;
        let from = core::mem::replace(&mut self.start, to);

        // SAFETY: This span is guaranteed to be valid
        let string = unsafe { self.backend.span_to_str(from, to) };

        Some((expect_valid_symbol(id), string, hash))
    }
}

/// An iterator over the interned symbols and their strings
pub struct Iter<'a, I: Intern + ?Sized, S> {
    inner: IterWithHashes<'a, I, S>,
}

impl<'a, I: Intern + ?Sized, S> Iter<'a, I, S> {
    #[cfg_attr(feature = "inline-more", inline)]
    fn new(backend: &'a StringBackend<I, S>) -> Self {
        Self {
            inner: IterWithHashes::new(backend),
        }
    }
}

impl<'a, I: Intern + ?Sized, S: Symbol> Iterator for Iter<'a, I, S>
where
    S: Symbol,
{
    type Item = (S, &'a I);

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
