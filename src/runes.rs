use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use std::ops::Index;

use crate::pointer;
use crate::{get_rune_cutoff_at_index, unwrap_indent, Result, Rune};

/// Represents a slice of bytes which can be automatically parsed into
/// a sequence of [Rune(s)](crate::Rune)
///
/// # Examples
///
///```
/// use utf8_rune::Runes;
/// let parts = Runes::new("👩🏻‍🚒👌🏿🧑🏽‍🚒👨‍🚒🌶️🎹💔🔥❤️‍🔥❤️‍🩹");
/// assert_eq!(
///     parts
///         .to_vec()
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
///
/// ```
/// use utf8_rune::Runes;
/// let runes = Runes::new("👌👌🏻👌🏼👌🏽👌🏾👌🏿");
///
/// assert_eq!(runes.rune_indexes(), vec![
///     (0, 4),
///     (4, 8),
///     (12, 8),
///     (20, 8),
///     (28, 8),
///     (36, 8),
/// ]);
/// assert_eq!(runes.len(), 6);
/// assert_eq!(runes[0], "👌");
/// assert_eq!(runes[1], "👌🏻");
/// assert_eq!(runes[2], "👌🏼");
/// assert_eq!(runes[3], "👌🏽");
/// assert_eq!(runes[4], "👌🏾");
/// assert_eq!(runes[5], "👌🏿");
/// ```

#[derive(Clone)]
pub struct Runes<'g> {
    pub(crate) ptr: *const u8,
    pub(crate) indexes: &'g [usize],
    pub(crate) length: usize,
    pub(crate) _marker: PhantomData<&'g usize>,
}
impl<'g> Default for Runes<'g> {
    fn default() -> Runes<'g> {
        Runes::empty().expect("memory allocation")
    }
}
impl<'g> Runes<'g> {
    pub fn new<T: Display>(input: T) -> Runes<'g> {
        Runes::allocate(&input)
            .expect(format!("allocate memory for Runes from {input}").as_str())
    }

    pub fn allocate<T: Display>(input: T) -> Result<Runes<'g>> {
        let input = input.to_string();
        let (ptr, length) = pointer::from_display(&input)?;
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
        Ok(Runes {
            ptr,
            indexes: indexes.leak(),
            length,
            _marker: PhantomData,
        })
    }

    pub fn empty() -> Result<Runes<'g>> {
        let length = 0;
        let ptr = pointer::create(length)?;
        Ok(Runes {
            ptr,
            length,
            indexes: &[],
            _marker: PhantomData,
        })
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

    pub fn len(&self) -> usize {
        self.indexes.len() -1
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
        let mut indexes = self.indexes.to_vec();
        if indexes.len() > 0 {
            indexes.pop();
        }
        indexes
    }

    pub fn rune_indexes(&self) -> Vec<(usize, usize)> {
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

    pub fn get(&self, index: usize) -> Option<Rune> {
        let indexes = self.rune_indexes();
        if index >= indexes.len() {
            None
        } else {
            let (index, length) = indexes[index];
            Some(Rune::from_raw_parts(unsafe { self.ptr.add(index) }, length))
        }
    }

    pub fn to_vec(&self) -> Vec<Rune> {
        let mut runes = Vec::<Rune>::new();
        for (index, length) in self.rune_indexes().into_iter() {
            runes.push(Rune::from_raw_parts(unsafe { self.ptr.add(index) }, length));
        }
        runes
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
impl<'g> Index<usize> for Runes<'g> {
    type Output = &'g str;

    fn index(&self, index: usize) -> &&'g str {
        if let Some(rune) = self.get(index) {
            unsafe { std::mem::transmute::<&&str, &&'g str>(&rune.as_str()) }
        } else {
            &""
        }
    }
}

#[cfg(test)]
mod test_runes {
    use crate::{Result, Runes};

    #[test]
    fn test_to_vec() -> Result<()> {
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
        Ok(())
    }
    #[test]
    fn test_length() -> Result<()> {
        let runes = Runes::new("👌👌🏻👌🏼👌🏽👌🏾👌🏿");
        let vec = runes.to_vec();

        assert_eq!(vec.len(), 6);
        assert_eq!(
            runes.rune_indexes(),
            vec![
                (0, 4),
                (4, 8),
                (12, 8),
                (20, 8),
                (28, 8),
                (36, 8),
            ]
        );
        assert_eq!(runes[0], "👌");
        assert_eq!(runes[1], "👌🏻");
        assert_eq!(runes[2], "👌🏼");
        assert_eq!(runes[3], "👌🏽");
        assert_eq!(runes[4], "👌🏾");
        assert_eq!(runes[5], "👌🏿");

        Ok(())
    }
}
