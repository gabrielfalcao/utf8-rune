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

pub mod internal;
pub(crate) use internal::{
    continuation_bytes_location, copy_ptr, dealloc_ptr, display_error, format_bytes,
    get_byte_at_index, get_byte_slice_of, get_valid_utf8_str_of, is_valid_utf8_str_of,
    new_ptr, slice_ptr_and_length_from_bytes, slice_ptr_and_length_from_display,
    unwrap_indent, DEFAULT_INDENT,
};

pub mod parts;
#[doc(inline)]
pub use parts::RuneParts;

pub mod heuristic;
#[doc(inline)]
pub use heuristic::{
    get_rune_cutoff_at_index, next_valid_cutoff, previous_valid_cutoff,
    split_at_first_rune, unexpected_continuation_byte_at_index_error,
};
