use core::fmt::Display;

use crate::{layout, Result};

//TODO: std::alloc => alloc::alloc
/// allocates a new, zero-initialized, raw pointer (i.e.: `*mut u8`) of N contiguous bytes where N=`length`
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
/// See [utf8_rune::pointer::create](crate::mem::create) for details.
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
/// See [utf8_rune::pointer::create](crate::mem::create) for details.
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
/// See [utf8_rune::pointer::create](crate::mem::create) for details.
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
/// See [utf8_rune::pointer::create](crate::mem::create) for details.
///
#[inline]
pub fn from_display<T: Display>(input: T) -> Result<(*mut u8, usize)> {
    Ok(from_slice(input.to_string().as_bytes())?)
}

#[inline]
pub fn get_byte_at_index<'g>(src: *const u8, index: usize) -> u8 {
    let byte = unsafe { src.add(index).read() };
    byte
}

#[inline]
pub fn get_byte_slice_of<'g>(src: *const u8, index: usize, count: usize) -> &'g [u8] {
    let bytes = unsafe { std::slice::from_raw_parts(src.add(index), count) };
    bytes
}

#[inline]
pub fn get_valid_utf8_str_of<'g>(
    src: *const u8,
    index: usize,
    count: usize,
) -> Option<&'g str> {
    let slice = get_byte_slice_of(src, index, count).to_vec();
    match std::str::from_utf8(&slice) {
        Ok(c) => Some(unsafe { std::mem::transmute::<&str, &'g str>(c) }),
        Err(_e_) => None,
    }
}

#[inline]
pub fn is_valid_utf8_str_of<'g>(src: *const u8, index: usize, count: usize) -> bool {
    get_valid_utf8_str_of(src, index, count).is_some()
}
