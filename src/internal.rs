use std::alloc::Layout;
use std::fmt::Display;
use std::iter::Iterator;

pub const DEFAULT_INDENT: usize = 4;
use crate::{ByteType, Error};

#[inline]
pub fn layout_of_size(size: usize) -> Layout {
    let actual_size = if size == 0 {
        1
    } else {
        size
    };
    match Layout::array::<u8>(actual_size) {
        Ok(layout) => layout,
        Err(e) => {
            let s = if actual_size != 1 {
                "s"
            } else {
                ""
            };
            panic!("failed to obtain a memory layout of {actual_size} u8 byte{s}: {e}")
        },
    }
}

#[inline]
pub fn new_ptr(size: usize) -> *mut u8 {
    let layout = layout_of_size(size);
    let ptr = unsafe {
        let ptr = std::alloc::alloc_zeroed(layout);
        if ptr.is_null() {
            std::alloc::handle_alloc_error(layout);
        }
        ptr
    };
    let ptr = ptr;
    for a in 0..size {
        unsafe {
            ptr.add(a).write(0);
        }
    }
    ptr
}
#[inline]
pub fn dealloc_ptr(ptr: *mut u8, length: usize) {
    if !ptr.is_null() {
        let layout = layout_of_size(length);
        unsafe {
            std::alloc::dealloc(ptr, layout);
        }
    }
}

#[inline]
pub fn copy_ptr(ptr: *const u8, length: usize) -> *mut u8 {
    let mut index = 0;
    let new_ptr = new_ptr(length);
    if length == 0 {
        return new_ptr;
    }
    while index < length {
        unsafe {
            new_ptr.add(index).write(ptr.add(index).read());
        }
        index += 1;
    }
    new_ptr
}
#[inline]
pub fn slice_ptr_and_length_from_bytes(bytes: &[u8]) -> (*mut u8, usize) {
    let bytes = bytes.to_vec();

    let ptr = new_ptr(bytes.len());
    let length = bytes.len();
    if length == 0 {
        return (ptr, 0);
    }
    for (i, c) in bytes.iter().enumerate() {
        unsafe {
            ptr.add(i).write(*c);
        }
    }
    (ptr, length)
}
#[inline]
pub fn slice_ptr_and_length_from_display<T: Display>(input: T) -> (*mut u8, usize) {
    slice_ptr_and_length_from_bytes(input.to_string().as_bytes())
}

#[inline]
pub fn get_byte_at_index<'g>(ptr: *const u8, index: usize) -> u8 {
    let byte = unsafe { ptr.add(index).read() };
    byte
}

#[inline]
pub fn get_byte_slice_of<'g>(ptr: *const u8, index: usize, count: usize) -> &'g [u8] {
    let bytes = unsafe { std::slice::from_raw_parts(ptr.add(index), count) };
    bytes
}
#[inline]
pub fn get_valid_utf8_str_of<'g>(
    ptr: *const u8,
    index: usize,
    count: usize,
) -> Option<&'g str> {
    let slice = get_byte_slice_of(ptr, index, count).to_vec();
    match std::str::from_utf8(&slice) {
        Ok(c) =>
            Some(unsafe { std::mem::transmute::<&str, &'g str>(c) }),
        Err(_e_) => None,
    }
}
#[inline]
pub fn is_valid_utf8_str_of<'g>(ptr: *const u8, index: usize, count: usize) -> bool {
    get_valid_utf8_str_of(ptr, index, count).is_some()
}

#[cfg(feature = "debug")]
pub fn format_bytes(bytes: &[u8], indent: Option<usize>) -> String {
    let indent = indent.unwrap_or_else(|| DEFAULT_INDENT);
    let padding = " ".repeat(indent);
    fn pad(byte: u8, indent: usize) -> String {
        let byte = byte.to_string();
        let pad = " ".repeat(indent - (1 - byte.len()));
        format!("{byte}{pad}")
    }
    format!(
        "[\n{}{}\n]",
        padding,
        bytes
            .iter()
            .map(Clone::clone)
            .map(|c| format!(
                "{}{}, // {:#?}",
                " ".repeat(indent + DEFAULT_INDENT),
                pad(c, indent),
                char::from(c)
            ))
            .collect::<Vec<String>>()
            .join("\n"),
    )
}
#[cfg(not(feature = "debug"))]
pub fn format_bytes(bytes: &[u8], indent: Option<usize>) -> String {
    let pad = " ".repeat(unwrap_indent(indent));
    format!(
        "{pad}[{}]",
        bytes
            .into_iter()
            .map(|c| c.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    )
}
pub fn unwrap_indent(indent: Option<usize>) -> usize {
    indent.unwrap_or_else(|| DEFAULT_INDENT)
}

#[inline]
pub fn continuation_bytes_location<'g>(
    ptr: *const u8,
    length: usize,
    index: usize,
) -> Option<(usize, ByteType)> {
    let shift: usize = (0xE2u8.leading_ones() & 0xEFu8.leading_ones()) as usize;
    if index + (shift - 1) < length {
        let zwj0 = get_byte_at_index(ptr, index);
        let zwj1 = get_byte_at_index(ptr, index + 1);
        let zwj2 = get_byte_at_index(ptr, index + 2);
        let next_rune_byte = get_byte_at_index(ptr, index + shift);
        let ty = if index + shift < length {
            ByteType::from(next_rune_byte)
        } else {
            ByteType::None
        };
        let tuple = (zwj0, zwj1, zwj2);
        match tuple {
            (0xE2, 0x80, 0x8D) => {
                let count = shift + ty.len();
                Some((count, ty))
            },
            (0xEF, 0xB8, 0x8F) => {
                let mut count = 0xEFu8.leading_ones() as usize;
                if let Some((next_count, _ty_)) =
                    continuation_bytes_location(ptr, length, index + shift)
                {
                    count += next_count
                }

                Some((count, ty))
            },
            _ => None,
        }
    } else {
        None
    }
}

#[cfg(feature = "debug")]
pub fn display_error<'e>(error: Error<'e>, ptr: *const u8, length: usize) {
    let filename = file!();
    let lineno = line!();
    eprintln!("{filename}:{lineno} {error}");
}
#[cfg(not(feature = "debug"))]
pub fn display_error<'e>(_error: Error<'e>, _ptr: *const u8, _length: usize) {}

#[cfg(test)]
mod tests {
    use crate::{
        continuation_bytes_location, slice_ptr_and_length_from_display, Result,
    };
    #[test]
    fn test_continuation_byte_0x200d() -> Result<()> {
        let (ptr, length) = slice_ptr_and_length_from_display("👩🏻‍🚒");

        assert_eq!(
            "👩🏻‍🚒".as_bytes().to_vec(),
            vec![
                0xF0, 0x9F, 0x91, 0xA9, 0xF0, 0x9F, 0x8F, 0xBB, 0xE2, 0x80, 0x8D, 0xF0,
                0x9F, 0x9A, 0x92
            ]
        );
        let location = continuation_bytes_location(ptr, length, 8);

        assert_eq!(location.is_some(), true);
        let (count, ty) = location.unwrap();
        assert_eq!(ty.len(), 4);
        assert_eq!(ty.byte(), 0xF0);
        assert_eq!(count, 7);

        Ok(())
    }
    #[test]
    fn test_continuation_byte_0xfe0f() -> Result<()> {
        let (ptr, length) = slice_ptr_and_length_from_display("❤️‍🔥");
        assert_eq!(
            "❤️‍🔥".as_bytes().to_vec(),
            vec![
                0xE2, 0x9D, 0xA4, 0xEF, 0xB8, 0x8F, 0xE2, 0x80, 0x8D, 0xF0, 0x9F, 0x94,
                0xA5
            ]
        );
        let location = continuation_bytes_location(ptr, length, 3);

        assert_eq!(location.is_some(), true);
        let (count, ty) = location.unwrap();
        assert_eq!(ty.len(), 3);
        assert_eq!(ty.byte(), 0xE2);
        assert_eq!(count, 10);

        Ok(())
    }
}
