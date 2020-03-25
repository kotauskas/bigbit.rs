# BigBit
![Crates.io link](https://img.shields.io/crates/v/bigbit "BigBit on Crates.io")

This is an implementation of the [BigBit standard][BigBitStd], used for representing arbitrarily large numbers and strings in a compact way. The only implementation provided by the author is [a Node.js implementation](https://github.com/bigbit/bigbitjs "BigBit.js on GitHub") — this crate aims to implement the functionality presented there with idiomatic Rust code.

In short, the specification describes 3 formats:
- **Head Byte** (***HB***), used for storing extremely large (but still finitely large) signed integers *and* decimal fractions without losing precision
- **Extended Head Byte** (***EHB***), used for storing arbitrarily large signed integers *and* decimal fractions without losing precision as long as sufficent storage space is provided.
- **Linked Bytes** (***LB***), used for storing arbitrarily large **unsigned integers**, mainly used directly inside the Extended Head Byte format to store the exponent and the number of additional coefficient bytes, but also useful for storing strings better than both UTF-8 and UTF-16.

Since this is a format parser, `#![no_std]` is enabled by default, meaning that `alloc` is the only dependency, allowing you to use this in a freestanding environment.

## State
Currently, not the entire BigBit standard is implemented. Here's a list of what's already done:
- Head Byte number storage
- Linked Bytes number storage
- Linked Bytes addition and subtraction (`Add`/`AddAssign` and `Sub`/`SubAssign`)

And here's a list of what's not finished just yet:
- Borrowed Linked Bytes (required for EHB) — a Linked Bytes number which doesn't own its contents and is a slice into an EHB number (will be added in 0.0.x)
- Creating HB/LB numbers from primitive integers and `f32`/`f64` (most likely will be added in 0.1.0)
- The Extended Header Byte format (will be added in 0.0.x)
- Arithmetic operations (addition, subtraction, multiplication and division are all defined by the BigBit standard), except for LB addition, which is already implemented; the main issue is dealing with the exponents (will mark the 1.0.0 release, might be partially added over the course of 0.x.x releases)
- Strings encoded using Linked Bytes (will be added in 0.0.x)
- `Debug` and `Display` formatting (i.e. converting the numbers either into a debugging-friendly representation as an array of bytes or a string representing the number in decimal scientific notation or full notation, as well as other numeric notations; simple `Debug` and `Display` decimal formatting will be added in 0.1.0 while the rest is planned for 1.0.0)
- **Tests** (planned for 0.1.0 but might be partially added earlier)

###### If you're wondering why all versions until `0.0.3` are yanked, these had a minor LB addition bug, so please upgrade if you haven't already.

[BigBitStd]: https://github.com/amitguptagwl/BigBit "BitBit specification on GitHub"
