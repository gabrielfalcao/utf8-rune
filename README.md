# UTF8 Rune

Lightweight crate that aims at being a building block for libraries
that work with UTF-8 data.

This crate provides the struct Rune which can thought of in some cases
as a drop-in replacement to Rust's char type.

This crate also provides a few low-level tools to work with raw
pointers of bytes and work with a sequence of bytes to produce valid
UTF-8 data.

The idea of Rune both borrows from and expands [Golang's notion](https://go.dev/) of rune
such that rather than representing one 32 bits integer, each
`utf8_rune::Rune` represents a set of bytes that, when displayed
together represent a single visible UTF-8 character.
