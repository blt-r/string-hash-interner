# String Hash Interner

A fork of [robbepop/string-interner](https://github.com/robbepop/string-interner).

The main point of this fork is that now the hashes of the strings interned are cached
and can be cheaply looked up with `Interner::get_hash` and `Interner::get_hash_unchecked`.

I only implemented this for the "String Backend", as that's the only backend I need.
Figuring out how to implement this for other backends or make an interface with ability
for backends to support this optionally is too complicated, so other backends were just removed.

This fork also makes the Interner generic over the type of strings interned. 
String types that are supported are: `str`, `CStr`, `OsStr`, `[u8]`, `[char]`.
