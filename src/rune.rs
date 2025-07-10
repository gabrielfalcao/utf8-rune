use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Deref;

use crate::pointer::{
    self, get_byte_slice_of,
};
use crate::{
    display_error, format_bytes, get_rune_cutoff_at_index, unwrap_indent, Result,
    DEFAULT_INDENT,
};

/// A Rune represents a single visible UTF-8 character. To handle contiguous bytes as multiple runes consider using [Runes](crate::Runes)
///
/// Examples
///
///```
/// use utf8_rune::Rune;
/// let rune = Rune::new("❤️");
/// assert_eq!(rune.len(), 6);
/// assert_eq!(rune.as_str(), "❤️");
/// assert_eq!(rune.as_bytes(), "❤️".as_bytes());
///```
///
///```
/// use utf8_rune::Rune;
/// let rune = Rune::new("👌🏽");
/// assert_eq!(rune.len(), 8);
/// assert_eq!(rune.as_str(), "👌🏽");
/// assert_eq!(rune.as_bytes(), "👌🏽".as_bytes());
///```

#[derive(Clone, Copy)]
pub struct Rune {
    pub(crate) ptr: *const u8,
    pub(crate) length: usize,
}

impl Default for Rune {
    fn default() -> Rune {
        Rune::empty().expect("memory allocation")
    }
}
impl Rune {
    pub fn new<T: Display>(input: T) -> Rune {
        Rune::allocate(&input)
            .expect(format!("allocate memory for Rune from {input}").as_str())
    }

    pub fn allocate<T: Display>(input: T) -> Result<Rune> {
        let (input_ptr, input_length) = pointer::from_display(input)?;
        match get_rune_cutoff_at_index(input_ptr, input_length, 0) {
            Ok(length) => {
                let ptr = pointer::create(length)?;
                for offset in 0..length {
                    unsafe {
                        ptr.add(offset)
                            .write(input_ptr.add(offset).read());
                    }
                }
                pointer::destroy(input_ptr, input_length)?;
                Ok(Rune { ptr, length })
            },
            Err(error) => {
                display_error(error, input_ptr, input_length);
                Ok(Rune::default())
            },
        }
    }

    pub fn empty() -> Result<Rune> {
        let length = 0;
        let ptr = pointer::create(length)?;
        Ok(Rune { ptr, length })
    }

    pub fn from_ptr_cutoff(
        input_ptr: *const u8,
        input_length: usize,
        index: usize,
    ) -> Result<Rune> {
        let cutoff = get_rune_cutoff_at_index(input_ptr, input_length, index)?;
        let length = cutoff - index;
        let ptr = pointer::create(length)?;

        for (index, byte) in get_byte_slice_of(input_ptr, index, length)
            .into_iter()
            .enumerate()
        {
            unsafe { ptr.add(index).write(*byte) }
        }
        Ok(Rune { ptr, length })
    }

    pub fn as_str<'g>(&self) -> &'g str {
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

    pub fn as_bytes<'g>(&self) -> &'g [u8] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.length) }
    }

    pub fn as_debug(&self, indent: Option<usize>) -> String {
        let indent = unwrap_indent(indent);
        format!(
            "Rune{{{}}}{}",
            self.as_str(),
            format_bytes(self.as_bytes(), Some(indent + DEFAULT_INDENT)),
        )
    }
}

impl From<&str> for Rune {
    fn from(s: &str) -> Rune {
        Rune::new(s)
    }
}

impl From<String> for Rune {
    fn from(s: String) -> Rune {
        Rune::new(s)
    }
}

impl From<&String> for Rune {
    fn from(s: &String) -> Rune {
        Rune::new(s)
    }
}

impl Display for Rune {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
impl Debug for Rune {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_debug(None))
    }
}

impl Deref for Rune {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        self.as_bytes()
    }
}

// impl Drop for Rune {
//     fn drop(&mut self) {
//         pointer::destroy(self.ptr, self.length)
//     }
// }

impl PartialEq<Rune> for Rune {
    fn eq(&self, other: &Rune) -> bool {
        self.as_bytes().eq(other.as_bytes())
    }
}
impl Eq for Rune {}

impl PartialOrd<Rune> for Rune {
    fn partial_cmp(&self, other: &Rune) -> Option<Ordering> {
        self.as_bytes().partial_cmp(&other.as_bytes())
    }
}
impl<'g> PartialOrd<&'g str> for Rune {
    fn partial_cmp(&self, other: &&'g str) -> Option<Ordering> {
        self.as_str().partial_cmp(other)
    }
}
impl<'g> PartialOrd<&'g [u8]> for Rune {
    fn partial_cmp(&self, other: &&'g [u8]) -> Option<Ordering> {
        self.as_bytes().partial_cmp(other)
    }
}
impl<'g> PartialOrd<Vec<u8>> for Rune {
    fn partial_cmp(&self, other: &Vec<u8>) -> Option<Ordering> {
        self.as_bytes().to_vec().partial_cmp(other)
    }
}
impl<'g> PartialOrd<&Vec<u8>> for Rune {
    fn partial_cmp(&self, other: &&Vec<u8>) -> Option<Ordering> {
        self.as_bytes().to_vec().partial_cmp(other)
    }
}

impl<'g> PartialEq<&'g str> for Rune {
    fn eq(&self, other: &&'g str) -> bool {
        self.as_str().eq(*other)
    }
}

impl<'g> PartialEq<&'g [u8]> for Rune {
    fn eq(&self, other: &&'g [u8]) -> bool {
        self.as_bytes().eq(*other)
    }
}
impl<'g> PartialEq<Vec<u8>> for Rune {
    fn eq(&self, other: &Vec<u8>) -> bool {
        self.as_bytes().to_vec().eq(other)
    }
}
impl<'g> PartialEq<&Vec<u8>> for Rune {
    fn eq(&self, other: &&Vec<u8>) -> bool {
        self.as_bytes().to_vec().eq(*other)
    }
}

impl Ord for Rune {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_bytes()
            .to_vec()
            .cmp(&other.as_bytes().to_vec())
    }
}

impl Hash for Rune {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_bytes().hash(state);
    }
}


#[cfg(test)]
mod test_rune {
    use crate::Rune;

    #[test]
    fn test_single_rune() {
        let rune = Rune::new("❤️");
        assert_eq!(rune.len(), 6);
        assert_eq!(rune.as_str(), "❤️");
        assert_eq!(rune.as_bytes(), "❤️".as_bytes());

        let rune = Rune::new("👌");
        assert_eq!(rune.len(), 4);
        assert_eq!(rune.as_str(), "👌");
        assert_eq!(rune.as_bytes(), "👌".as_bytes());

        let rune = Rune::new("👌🏻");
        assert_eq!(rune.len(), 8);
        assert_eq!(rune.as_str(), "👌🏻");
        assert_eq!(rune.as_bytes(), "👌🏻".as_bytes());

        let rune = Rune::new("👌🏼");
        assert_eq!(rune.len(), 8);
        assert_eq!(rune.as_str(), "👌🏼");
        assert_eq!(rune.as_bytes(), "👌🏼".as_bytes());

        let rune = Rune::new("👌🏽");
        assert_eq!(rune.len(), 8);
        assert_eq!(rune.as_str(), "👌🏽");
        assert_eq!(rune.as_bytes(), "👌🏽".as_bytes());

        let rune = Rune::new("👌🏾");
        assert_eq!(rune.len(), 8);
        assert_eq!(rune.as_str(), "👌🏾");
        assert_eq!(rune.as_bytes(), "👌🏾".as_bytes());

        let rune = Rune::new("👌🏿");
        assert_eq!(rune.len(), 8);
        assert_eq!(rune.len(), 8);
        assert_eq!(rune.as_str(), "👌🏿");
        assert_eq!(rune.as_bytes(), "👌🏿".as_bytes());
    }

    #[test]
    fn test_from_multiple_runes() {
        let rune = Rune::new("👌👌🏻👌🏼👌🏽👌🏾👌🏿");
        assert_eq!(rune.len(), 4);
        assert_eq!(rune.as_str(), "👌");
        assert_eq!(rune.as_bytes(), "👌".as_bytes());

        let rune = Rune::new("👌🏻👌🏼👌🏽👌🏾👌🏿");
        assert_eq!(rune.len(), 8);
        assert_eq!(rune.as_str(), "👌🏻");
        assert_eq!(rune.as_bytes(), "👌🏻".as_bytes());

        let rune = Rune::new("👌🏼👌🏽👌🏾👌🏿");
        assert_eq!(rune.len(), 8);
        assert_eq!(rune.as_str(), "👌🏼");
        assert_eq!(rune.as_bytes(), "👌🏼".as_bytes());

        let rune = Rune::new("👌🏽👌🏾👌🏿");
        assert_eq!(rune.len(), 8);
        assert_eq!(rune.as_str(), "👌🏽");
        assert_eq!(rune.as_bytes(), "👌🏽".as_bytes());

        let rune = Rune::new("👌🏾👌🏿");
        assert_eq!(rune.len(), 8);
        assert_eq!(rune.as_str(), "👌🏾");
        assert_eq!(rune.as_bytes(), "👌🏾".as_bytes());

        let rune = Rune::new("👌🏿");
        assert_eq!(rune.len(), 8);
        assert_eq!(rune.as_str(), "👌🏿");
        assert_eq!(rune.as_bytes(), "👌🏿".as_bytes());
    }
}
