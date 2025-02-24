use core::{ffi::CStr, fmt::Debug, hash::Hash};

/// Trait implemented by different types of strings that can be interned.
///
/// # Safety
///
/// It should be alwas valid to reinterpret bytes of `Self` as `&[Self::Primitive]`
/// using [`Intern::as_bytes`].
///
/// It should be valid to reinterpret bytes copied from [`Intern::as_bytes`].
/// as `&Self` using [`Intern::from_bytes`]. Even if they were moved in memory.
pub unsafe trait Intern: Hash + PartialEq + Eq {
    /// A primitive type that has the same alignment as `Self`.
    type Primitive: Sized + Copy + Debug;

    fn as_bytes(&self) -> &[Self::Primitive];

    /// # Safety
    ///
    /// See [Safety section](Intern#safety) in the trait doc.
    unsafe fn from_bytes(bytes: &[Self::Primitive]) -> &Self;
}

unsafe impl Intern for str {
    type Primitive = u8;

    fn as_bytes(&self) -> &[u8] {
        str::as_bytes(self)
    }

    unsafe fn from_bytes(bytes: &[u8]) -> &Self {
        // SAFETY: Calling this function is only valid with bytes obtained from `Self::as_bytes`.
        unsafe { core::str::from_utf8_unchecked(bytes) }
    }
}

unsafe impl Intern for CStr {
    type Primitive = u8;

    fn as_bytes(&self) -> &[u8] {
        CStr::to_bytes_with_nul(self)
    }

    unsafe fn from_bytes(bytes: &[u8]) -> &Self {
        // SAFETY: Calling this function is only valid with bytes obtained from `Self::as_bytes`.
        unsafe { CStr::from_bytes_with_nul_unchecked(bytes) }
    }
}

unsafe impl Intern for [u8] {
    type Primitive = u8;

    fn as_bytes(&self) -> &[u8] {
        self
    }

    unsafe fn from_bytes(bytes: &[u8]) -> &Self {
        bytes
    }
}

unsafe impl Intern for [char] {
    type Primitive = char;

    fn as_bytes(&self) -> &[char] {
        self
    }

    unsafe fn from_bytes(bytes: &[char]) -> &Self {
        bytes
    }
}

#[cfg(feature = "std")]
mod std_impls {
    use super::Intern;
    use std::ffi::OsStr;

    unsafe impl Intern for OsStr {
        type Primitive = u8;

        fn as_bytes(&self) -> &[u8] {
            OsStr::as_encoded_bytes(self)
        }

        unsafe fn from_bytes(bytes: &[u8]) -> &Self {
            // SAFETY: Calling this function is only valid with bytes obtained from `Self::as_bytes`.
            unsafe { OsStr::from_encoded_bytes_unchecked(bytes) }
        }
    }
}

// TODO: add impl for `[std::ascii::Char]` when stable.
