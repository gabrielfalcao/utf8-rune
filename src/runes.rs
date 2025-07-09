use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use std::ops::Deref;

use crate::{
    get_rune_cutoff_at_index, get_valid_utf8_str_of, slice_ptr_and_length_from_display,
    unwrap_indent, Result, Rune,
};

/// Represents a slice of bytes which can be automatically parsed into
/// a sequence of [Rune(s)](crate::Rune)
///
/// Example
///
///```
/// use utf8_rune::{Runes};
/// let parts = Runes::new("👩🏻‍🚒👌🏿🧑🏽‍🚒👨‍🚒🌶️🎹💔🔥❤️‍🔥❤️‍🩹");
/// assert_eq!(
///     parts
///         .runes().unwrap_or_default()
///         .iter()
///         .map(|rune| rune.to_string())
///         .collect::<Vec<String>>(),
///     vec![
///         "👩🏻‍🚒",
///         "👌🏿",
///         "🧑🏽‍🚒",
///         "👨‍🚒",
///         "🌶️",
///         "🎹",
///         "💔",
///         "🔥",
///         "❤️‍🔥",
///         "❤️‍🩹",
///     ]
/// );
///```
#[derive(Clone)]
pub struct Runes<'g> {
    pub(crate) ptr: *const u8,
    pub(crate) indexes: Vec<usize>,
    pub(crate) length: usize,
    pub(crate) _marker: PhantomData<&'g usize>,
}
impl<'g> Runes<'g> {
    pub fn new<T: Display>(input: T) -> Runes<'g> {
        let input = input.to_string();
        let (ptr, length) = slice_ptr_and_length_from_display(&input);
        let mut cutoff: usize = 0;
        let mut indexes = vec![cutoff];
        while cutoff < length {
            match get_rune_cutoff_at_index(ptr, length, cutoff) {
                Ok(next) => {
                    indexes.push(next);
                    cutoff = next;
                },
                Err(_) => break,
            }
        }

        Runes {
            ptr,
            indexes,
            length,
            _marker: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.indexes.len()
    }

    pub fn as_str(&self) -> &'g str {
        let mut offset = self.length;
        loop {
            if let Ok(slice) = std::str::from_utf8(unsafe {
                std::slice::from_raw_parts(self.ptr, offset)
            }) {
                break slice;
            }
            if offset > 0 {
                offset -= 1;
            } else {
                break "";
            }
        }
    }

    pub fn as_bytes(&self) -> &'g [u8] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.length) }
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn as_debug(&self, indent: Option<usize>) -> String {
        let indent = unwrap_indent(indent);
        let length = self.len();
        format!(
            "Runes{{{}}}",
            [format!("length: {length}"),]
                .iter()
                .map(|c| {
                    let padding = " ".repeat(indent);
                    format!("{padding}{c}")
                })
                .collect::<Vec<String>>()
                .join("\n")
        )
    }

    pub fn indexes(&self) -> Vec<usize> {
        let mut indexes = self.indexes.clone();
        if indexes.len() > 0 {
            indexes.pop();
        }
        indexes
    }

    pub fn slice_indexes(&self) -> Vec<(usize, usize)> {
        self.indexes()
            .into_iter()
            .map(|index| {
                let next = get_rune_cutoff_at_index(self.ptr, self.length, index)
                    .unwrap_or_default();
                let length = if next >= index {
                    next - index
                } else {
                    0
                };
                (index, length)
            })
            .filter(|(_, length)| *length > 0)
            .collect()
    }

    pub fn runes(&self) -> Result<Vec<Rune>> {
        let mut runes = Vec::<Rune>::new();
        for cutoff in self.indexes().into_iter() {
            runes.push(Rune::from_ptr_cutoff(self.ptr, self.length, cutoff)?);
        }
        Ok(runes)
    }
}
impl<'g> From<&str> for Runes<'g> {
    fn from(s: &str) -> Runes<'g> {
        Runes::new(s)
    }
}

impl<'g> From<String> for Runes<'g> {
    fn from(s: String) -> Runes<'g> {
        Runes::new(s)
    }
}

impl<'g> From<&String> for Runes<'g> {
    fn from(s: &String) -> Runes<'g> {
        Runes::new(s)
    }
}

impl<'g> Display for Runes<'g> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
impl<'g> Debug for Runes<'g> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_debug(None))
    }
}
impl<'g> Deref for Runes<'g> {
    type Target = [&'g str];

    fn deref(&self) -> &[&'g str] {
        let runes = self
            .slice_indexes()
            .into_iter()
            .map(|(index, count)| get_valid_utf8_str_of(self.ptr, index, count))
            .filter(|c| c.is_some())
            .map(|c| c.unwrap())
            .collect::<Vec<&'g str>>();
        unsafe { std::mem::transmute::<&[&str], &'g [&'g str]>(&runes) }
    }
}
