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


# Examples


## `utf8_rune::Rune`

```rust
use utf8_rune::Rune;

let rune = Rune::new("👩🏻‍🚒");

assert_eq!(rune.len(), 15);
assert_eq!(rune.as_str(), "👩🏻‍🚒");
assert_eq!(rune.as_bytes(), "👩🏻‍🚒".as_bytes());
assert_eq!(rune.as_bytes(), *&rune);
```

## `utf8_rune::Runes`

```rust
use utf8_rune::Runes;

let parts = Runes::new("👩🏻‍🚒👌🏿🧑🏽‍🚒👨‍🚒🌶️🎹💔🔥❤️‍🔥❤️‍🩹");

assert_eq!(
    parts
        .to_vec()
        .iter()
        .map(|rune| rune.to_string())
        .collect::<Vec<String>>(),
    vec![
        "👩🏻‍🚒",
        "👌🏿",
        "🧑🏽‍🚒",
        "👨‍🚒",
        "🌶️",
        "🎹",
        "💔",
        "🔥",
        "❤️‍🔥",
        "❤️‍🩹",
    ]
);
```

```rust
use utf8_rune::Runes;

let runes = Runes::new("👌👌🏻👌🏼👌🏽👌🏾👌🏿");

assert_eq!(runes.rune_indexes(), vec![
    (0, 4),
    (4, 8),
    (12, 8),
    (20, 8),
    (28, 8),
    (36, 8),
]);

assert_eq!(runes.len(), 6);
assert_eq!(runes[0], "👌");
assert_eq!(runes[1], "👌🏻");
assert_eq!(runes[2], "👌🏼");
assert_eq!(runes[3], "👌🏽");
assert_eq!(runes[4], "👌🏾");
assert_eq!(runes[5], "👌🏿");
```

## `utf8_rune::RuneParts`

```rust
use utf8_rune::{RuneParts, Rune, Runes};

let parts = RuneParts::new("👌👌🏻👌🏼👌🏽👌🏾👌🏿");

assert_eq!(parts.len(), 44);
assert_eq!(parts.as_str(), "👌👌🏻👌🏼👌🏽👌🏾👌🏿");
assert_eq!(parts.as_bytes(), "👌👌🏻👌🏼👌🏽👌🏾👌🏿".as_bytes());

let runes = parts.into_runes();

assert_eq!(runes.len(), 6);
assert_eq!(runes[0], "👌");
assert_eq!(runes[1], "👌🏻");
assert_eq!(runes[2], "👌🏼");
assert_eq!(runes[3], "👌🏽");
assert_eq!(runes[4], "👌🏾");
assert_eq!(runes[5], "👌🏿");
```

## `utf8_rune::heuristic`

```rust
use utf8_rune::get_rune_cutoff_at_index;

let bytes = "👩🏻‍🚒👌🏿🧑🏽‍🚒👨‍🚒🌶️🎹💔🔥❤️‍🔥❤️‍🩹".as_bytes();
let length = bytes.len();
let ptr = bytes.as_ptr();

let index = 56;
let cutoff = get_rune_cutoff_at_index(ptr, length, index).unwrap();
assert_eq!(std::str::from_utf8(&bytes[index..cutoff]), Ok("🎹"));
```
