# BigBit
[![Crates.io](https://img.shields.io/crates/v/bigbit)](https://crates.io/crates/bitbit "BigBit on Crates.io")
[![Docs.rs](https://img.shields.io/badge/documentation-docs.rs-informational)](https://docs.rs/bigbit "BigBit on Docs.rs")
[![Build Status](https://github.com/kotauskas/bigbit.rs/workflows/Build/badge.svg)](https://github.com/kotauskas/bigbit.rs/actions "GitHub Actions page for BigBit")

This is an implementation of the [BigBit standard][BigBitStd], used for representing arbitrarily large numbers and strings in a compact way. The only implementation provided by the author is [a Node.js implementation](https://github.com/bigbit/bigbitjs "BigBit.js on GitHub") — this crate aims to implement the functionality presented there with idiomatic Rust code.

In short, the specification describes 3 formats:
- **Head Byte** (***HB***), used for storing extremely large (but still finitely large) signed integers *and* decimal fractions without losing precision
- **Extended Head Byte** (***EHB***), used for storing arbitrarily large signed integers *and* decimal fractions without losing precision as long as sufficent storage space is provided.
- **Linked Bytes** (***LB***), used for storing arbitrarily large **unsigned integers**, mainly used directly inside the Extended Head Byte format to store the exponent and the number of additional coefficient bytes, but also useful for storing strings better than both UTF-8 and UTF-16.

Since this is a format parser, `#![no_std]` is enabled by default, meaning that `alloc` is the only dependency, allowing you to use this in a freestanding environment.

## State
Currently, not the entire BigBit standard is implemented, and **the crate is not ready for use in production just yet**. There are also **no stability guarantees whatsoever**. Here's a list of what's already done:
- Head Byte number storage (not really finished, just a stub)
- Linked Bytes number storage and arithmetic
- Converting Linked Bytes to and from primitive integers
- Displaying Linked Bytes numbers in binary, octal, decimal and hexadecimal using formatters as well as other bases (arbitrary from 2 to 36) using a dedicated method
- Borrowed Linked Bytes (required for EHB) — a Linked Bytes number which doesn't own its contents and is a slice into an EHB number (still a stub)

And here's a list of what's not finished just yet:
- Creating \[E\]HB numbers from primitive integers and `f32`/`f64` (most likely will be added in 0.1.0)
- The Extended Header Byte format (will be added in 0.0.x)
- Arithmetic operations (addition, subtraction, multiplication and division are all defined by the BigBit standard) for \[E\]HB; the main issue is dealing with the exponents (will mark the 1.0.0 release, might be partially added over the course of 0.x.x releases)
- Strings encoded using Linked Bytes (will be added in 0.0.x)
- `Debug` and `Display` formatting for \[E\]HB (i.e. converting the numbers either into a debugging-friendly representation as an array of bytes or a string representing the number in decimal scientific notation or full notation, as well as other numeric notations; simple `Debug` and `Display` decimal formatting will be added in 0.1.0 while the rest is planned for 1.0.0)
- **Tests** (planned for 0.1.0 but might be partially added earlier)

## Changelog
The full version history can be found [here][changelog].

[BigBitStd]: https://github.com/amitguptagwl/BigBit "BitBit specification on GitHub"
[changelog]: https://github.com/kotauskas/bigbit.rs/releases " "
