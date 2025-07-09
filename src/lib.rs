pub mod errors;
pub use errors::{Error, Result};

pub mod byte_type;
pub use byte_type::ByteType;

pub mod rune;
pub use rune::Rune;

pub mod runes;
pub use runes::Runes;

pub mod internal;
pub use internal::{
    continuation_bytes_location, copy_ptr, dealloc_ptr, display_error, format_bytes,
    get_byte_at_index, get_byte_slice_of, is_valid_utf8_str_of, layout_of_size,
    new_ptr, slice_ptr_and_length_from_bytes, slice_ptr_and_length_from_display,
    unwrap_indent, DEFAULT_INDENT,
};
pub use ByteType::{Ascii, Continuation, FourOrMore, One, Three, Two};

pub mod parts;
pub use parts::RuneParts;

pub mod heuristic;
pub use heuristic::{
    get_rune_cutoff_at_index, next_valid_cutoff, previous_valid_cutoff,
    split_at_first_rune, unexpected_continuation_byte_at_index_error,
};
