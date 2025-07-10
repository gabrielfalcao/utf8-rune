mod errors;
#[doc(inline)]
pub use errors::{Error, Result};

mod byte_type;
#[doc(inline)]
pub use byte_type::ByteType;

mod rune;
#[doc(inline)]
pub use rune::Rune;

mod runes;
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

mod parts;
#[doc(inline)]
pub use parts::RuneParts;

mod heuristic;
#[doc(inline)]
pub use heuristic::{
    continuation_bytes_location, get_rune_cutoff_at_index, split_at_first_rune,
};

#[cfg(not(feature = "pointer"))]
pub(crate) mod mem;
#[cfg(feature = "pointer")]
pub mod mem;
#[cfg(not(feature = "pointer"))]
pub(crate) use mem::layout;
#[cfg(feature = "pointer")]
pub use mem::{layout, MemoryError};
