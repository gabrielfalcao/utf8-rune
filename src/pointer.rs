use core::fmt::Display;

use crate::{layout, Result};

//TODO: std::alloc => alloc::alloc
/// allocates a new, zero-initialized, raw pointer (i.e.: `*mut u8`) of N contiguous bytes where N=`length`
///
/// Example
///
/// ```
/// use utf8_rune::pointer::{self, from_slice};
/// let (ptr, length) = from_slice(b"bytes").unwrap();
/// pointer::destroy(ptr, length).unwrap();
/// ```
#[inline]
pub fn create(length: usize) -> Result<*mut u8> {
    let layout = layout(length)?;
    let src = unsafe {
        let src = std::alloc::alloc_zeroed(layout);
        if src.is_null() {
            std::alloc::handle_alloc_error(layout);
        }
        src
    };
    let src = src;
    for a in 0..length {
        unsafe {
            src.add(a).write(0);
        }
    }
    Ok(src)
}

/// deallocates a raw pointer (i.e.: `*mut u8`) of N contiguous bytes where N=`length`
///
/// See [utf8_rune::pointer::create](crate::pointer::create) for details.
///
/// Example
///
/// ```
/// use utf8_rune::pointer::{self, from_display};
/// let (ptr, length) = from_display("bytes").unwrap();
/// pointer::destroy(ptr, length).unwrap();
/// ```
///
#[inline]
pub fn destroy(src: *mut u8, length: usize) -> Result<()> {
    if !src.is_null() {
        let layout = layout(length)?;
        unsafe {
            std::alloc::dealloc(src, layout);
        }
    }
    Ok(())
}

/// copies the memory contents of `src` into a newly allocated pointer
///
/// See [utf8_rune::pointer::create](crate::pointer::create) for details.
///
/// Example
///
/// ```
/// use utf8_rune::pointer::{self, from_display, get_byte_at_index, copy};
/// let (ptr, length) = from_display("bytes").unwrap();
/// let other_ptr = copy(ptr, length).unwrap();
/// assert_eq!(get_byte_at_index(ptr, 2), b't');
/// pointer::destroy(ptr, length).unwrap();
/// pointer::destroy(other_ptr, length).unwrap();
/// ```
///
#[inline]
pub fn copy(src: *const u8, length: usize) -> Result<*mut u8> {
    let mut index = 0;
    let alloc = create(length)?;
    if length == 0 {
        Ok(alloc)
    } else {
        while index < length {
            unsafe {
                alloc.add(index).write(src.add(index).read());
            }
            index += 1;
        }
        Ok(alloc)
    }
}

/// copies the memory from the given slice of bytes into a newly allocated pointer
///
/// See [utf8_rune::pointer::create](crate::pointer::create) for details.
///
/// Example
///
/// ```
/// use utf8_rune::pointer::{self, from_slice, get_byte_at_index};
/// let (ptr, length) = from_slice(b"bytes").unwrap();
/// assert_eq!(get_byte_at_index(ptr, 2), b't');
/// pointer::destroy(ptr, length).unwrap();
/// ```
///
#[inline]
pub fn from_slice(bytes: &[u8]) -> Result<(*mut u8, usize)> {
    let bytes = bytes.to_vec();

    let src = create(bytes.len())?;
    let length = bytes.len();
    if length == 0 {
        Ok((src, 0))
    } else {
        for (i, c) in bytes.iter().enumerate() {
            unsafe {
                src.add(i).write(*c);
            }
        }
        Ok((src, length))
    }
}

/// copies the memory from the underlying slice of bytes of the given
/// `input` into a newly allocated pointer.
///
/// See [utf8_rune::pointer::create](crate::pointer::create) for details.
///
/// Example
///
/// ```
/// use utf8_rune::pointer::{self, from_display, get_byte_at_index};
/// let (ptr, length) = from_display("bytes").unwrap();
/// assert_eq!(get_byte_at_index(ptr, 2), b't');
/// pointer::destroy(ptr, length).unwrap();
/// ```
///
#[inline]
pub fn from_display<T: Display>(input: T) -> Result<(*mut u8, usize)> {
    Ok(from_slice(input.to_string().as_bytes())?)
}

/// retrieves a byte from contiguous memory area
///
/// Example
///
/// ```
/// use utf8_rune::pointer::get_byte_at_index;
/// let ptr = "bytes".as_bytes().as_ptr();
/// assert_eq!(get_byte_at_index(ptr, 2), b't');
/// ```
#[inline]
pub fn get_byte_at_index<'g>(src: *const u8, index: usize) -> u8 {
    let byte = unsafe { src.add(index).read() };
    byte
}

/// retrieves a slice of N bytes from contiguous memory area where `N=count` starting at `index`
///
/// Example
///
/// ```
/// use utf8_rune::pointer::get_byte_slice_of;
/// let ptr = "UNICODE".as_bytes().as_ptr();
/// assert_eq!(get_byte_slice_of(ptr, 2, 3), b"ICO");
/// ```
#[inline]
pub fn get_byte_slice_of<'g>(src: *const u8, index: usize, count: usize) -> &'g [u8] {
    let bytes = unsafe { std::slice::from_raw_parts(src.add(index), count) };
    bytes
}

/// retrieves `Some()` valid str slice of N bytes from contiguous
/// memory area or `None` if the resulting sequence of bytes is not
/// valid UTF-8.
///
/// Example
///
/// ```
/// use utf8_rune::pointer::get_valid_utf8_str_of;
/// let ptr = "UNICODE".as_bytes().as_ptr();
/// assert_eq!(get_valid_utf8_str_of(ptr, 2, 3), Some("ICO"));
/// ```
#[inline]
pub fn get_valid_utf8_str_of<'g>(
    src: *const u8,
    index: usize,
    count: usize,
) -> Option<&'g str> {
    std::str::from_utf8(get_byte_slice_of(src, index, count)).ok()
}

/// returns `true` if the sequence of `count` bytes from `index` is a
/// valid UTF-8 sequence.
///
/// Example
///
/// ```
/// use utf8_rune::pointer::is_valid_utf8_str_of;
/// let ptr = "UNICODE".as_bytes().as_ptr();
/// assert_eq!(is_valid_utf8_str_of(ptr, 2, 3), true);
/// ```
#[inline]
pub fn is_valid_utf8_str_of<'g>(src: *const u8, index: usize, count: usize) -> bool {
    get_valid_utf8_str_of(src, index, count).is_some()
}
