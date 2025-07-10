pub mod errors;
#[doc(inline)]
pub use errors::{Error, Result};

pub mod byte_type;
#[doc(inline)]
pub use byte_type::ByteType;

pub mod rune;
#[doc(inline)]
pub use rune::Rune;

pub mod runes;
#[doc(inline)]
pub use runes::Runes;

pub mod pointer;

#[cfg(feature = "pointer")]
pub use pointer::{
    copy, create, destroy, from_display, from_slice, get_byte_at_index,
    get_byte_slice_of, get_valid_utf8_str_of, is_valid_utf8_str_of,
};
#[cfg(not(feature = "pointer"))]
#[allow(unused_imports)]
pub(crate) use pointer::{
    copy, create, destroy, from_display, from_slice, get_byte_at_index,
    get_byte_slice_of, get_valid_utf8_str_of, is_valid_utf8_str_of,
};

pub mod internal;
pub(crate) use internal::{display_error, format_bytes, unwrap_indent, DEFAULT_INDENT};

pub mod parts;
#[doc(inline)]
pub use parts::RuneParts;

pub mod heuristic;
#[doc(inline)]
pub use heuristic::{
    continuation_bytes_location, get_rune_cutoff_at_index, next_valid_cutoff,
    previous_valid_cutoff, split_at_first_rune,
};

pub mod mem;
#[cfg(not(feature = "pointer"))]
pub(crate) use mem::layout;
#[cfg(feature = "pointer")]
pub use mem::{layout, MemoryError};
