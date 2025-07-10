use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use crate::pointer::{
    self,
};
use crate::{
    display_error, format_bytes, get_rune_cutoff_at_index, unwrap_indent, Result, Rune, Runes,
    DEFAULT_INDENT,
};


///
/// Represents a memory area with contiguous bytes that serves as
/// building block for [Runes](crate::Runes) and [Rune](crate::Rune).
/// # Examples
///
///```
/// use utf8_rune::{RuneParts, Rune, Runes};
/// let parts = RuneParts::new("👌👌🏻👌🏼👌🏽👌🏾👌🏿");
/// assert_eq!(parts.len(), 44);
/// assert_eq!(parts.as_str(), "👌👌🏻👌🏼👌🏽👌🏾👌🏿");
/// assert_eq!(parts.as_bytes(), "👌👌🏻👌🏼👌🏽👌🏾👌🏿".as_bytes());
///
/// let runes = parts.into_runes();
/// assert_eq!(runes.len(), 6);
/// assert_eq!(runes[0], "👌");
/// assert_eq!(runes[1], "👌🏻");
/// assert_eq!(runes[2], "👌🏼");
/// assert_eq!(runes[3], "👌🏽");
/// assert_eq!(runes[4], "👌🏾");
/// assert_eq!(runes[5], "👌🏿");
///```
///
#[derive(Clone, Copy)]
pub struct RuneParts {
    pub ptr: *const u8,
    pub length: usize,
}
impl RuneParts {
    pub fn from_raw_parts(ptr: *const u8, length: usize) -> RuneParts {
        RuneParts { ptr, length }
    }

    pub fn new<T: Display>(input: T) -> RuneParts {
        RuneParts::allocate(&input)
            .expect(format!("allocate memory for RuneParts from {input}").as_str())
    }

    pub fn into_runes<'g>(self) -> Runes<'g> {
        let ptr = self.ptr;
        let length = self.length;
        let indexes = self.indexes().leak();
        Runes {
            ptr,
            length,
            indexes,
            _marker: PhantomData,
        }
    }

    pub fn allocate<T: Display>(input: T) -> Result<RuneParts> {
        let input = input.to_string();
        let (ptr, length) = pointer::from_display(&input)?;
        Ok(RuneParts { ptr, length })
    }

    pub fn rune(&self) -> Option<Rune> {
        match self.rune_at_index(0) {
            Ok(rune) => Some(rune),
            Err(error) => {
                display_error(error, self.ptr, self.length);
                None
            },
        }
    }

    pub fn indexes(&self) -> Vec<usize> {
        let mut cutoff = 0usize;
        let mut indexes = vec![cutoff];
        while cutoff < self.length {
            match get_rune_cutoff_at_index(self.ptr, self.length, cutoff) {
                Ok(next) => {
                    indexes.push(next);
                    cutoff = next;
                },
                Err(_) => break,
            }
        }
        indexes
    }

    pub fn rune_at_index(&self, index: usize) -> Result<Rune> {
        let cutoff = get_rune_cutoff_at_index(self.ptr, self.length, index)?;
        let length = cutoff - index;
        let ptr = pointer::create(length)?;
        for offset in index..cutoff {
            unsafe {
                ptr.add(offset - index)
                    .write(self.ptr.add(offset).read());
            }
        }
        Ok(Rune { ptr, length })
    }

    pub fn runes(&self) -> Result<Vec<Rune>> {
        let mut runes = Vec::<Rune>::new();
        let mut index = 0;
        while index < self.length {
            let cutoff = match get_rune_cutoff_at_index(self.ptr, self.length, index) {
                Ok(cutoff) => cutoff,
                #[allow(unused_variables)]
                Err(e) => {
                    #[cfg(feature = "debug")]
                    {
                        eprintln!();
                        dbg!(e);
                    }
                    break;
                },
            };
            let length = cutoff - index;
            let ptr = pointer::create(length)?;
            for offset in 0..length {
                unsafe {
                    ptr.add(offset)
                        .write(self.ptr.add(index + offset).read());
                }
            }
            runes.push(Rune { ptr, length });
            index = cutoff;
        }
        Ok(runes)
    }

    pub fn len(&self) -> usize {
        self.length
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

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn as_debug(&self, indent: Option<usize>) -> String {
        let indent = unwrap_indent(indent);
        let length = self.length;
        format!(
            "RuneParts{{{}}}",
            [
                if let Some(rune) = self.rune() {
                    format!("rune: {:#?}", rune.as_debug(Some(indent + DEFAULT_INDENT)))
                } else {
                    String::new()
                },
                format!("length: {length}"),
                format!(
                    "remaining_bytes: {}",
                    format_bytes(self.as_bytes(), Some(indent + DEFAULT_INDENT))
                ),
            ]
            .into_iter()
            .filter(|c| !c.is_empty())
            .map(|c| {
                let padding = " ".repeat(indent);
                format!("{padding}{c}")
            })
            .collect::<Vec<String>>()
            .join("\n")
        )
    }
}
impl From<&str> for RuneParts {
    fn from(s: &str) -> RuneParts {
        RuneParts::new(s)
    }
}

impl From<String> for RuneParts {
    fn from(s: String) -> RuneParts {
        RuneParts::new(s)
    }
}

impl From<&String> for RuneParts {
    fn from(s: &String) -> RuneParts {
        RuneParts::new(s)
    }
}

impl Display for RuneParts {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
impl Debug for RuneParts {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_debug(None))
    }
}

#[cfg(test)]
mod test_parts {
    use crate::{Rune, RuneParts};

    #[test]
    fn test_rune_at_index_error() {
        let parts = RuneParts::new("👌👌🏻👌🏼👌🏽👌🏾👌🏿");
        {
            let result = parts.rune_at_index(1); // Ok("👌")
            assert!(result.is_err());
            let err = result.err().unwrap();
            assert_eq!(err.previous_valid_cutoff(), Some(0));
            assert_eq!(err.next_valid_cutoff(), Some(4));
        }
        {
            let result = parts.rune_at_index(5); // Ok("👌🏻")
            assert!(result.is_err());
            let err = result.err().unwrap();
            assert_eq!(err.previous_valid_cutoff(), Some(4));
            assert_eq!(err.next_valid_cutoff(), Some(8));
        }
        {
            let result = parts.rune_at_index(13); // Ok("👌🏼")
            assert!(result.is_err());
            let err = result.err().unwrap();
            assert_eq!(err.previous_valid_cutoff(), Some(12));
            assert_eq!(err.next_valid_cutoff(), Some(16));
        }
        {
            let result = parts.rune_at_index(21); // Ok("👌🏽")
            assert!(result.is_err());
            let err = result.err().unwrap();
            assert_eq!(err.previous_valid_cutoff(), Some(20));
            assert_eq!(err.next_valid_cutoff(), Some(24));
        }
        {
            let result = parts.rune_at_index(29); // Ok("👌🏾")
            assert!(result.is_err());
            let err = result.err().unwrap();
            assert_eq!(err.previous_valid_cutoff(), Some(28));
            assert_eq!(err.next_valid_cutoff(), Some(32));
        }

        {
            let result = parts.rune_at_index(37); // Ok("👌🏿")
            assert!(result.is_err());
            let err = result.err().unwrap();
            assert_eq!(err.previous_valid_cutoff(), Some(36));
            assert_eq!(err.next_valid_cutoff(), Some(40));
        }
    }

    #[test]
    fn test_new_single_rune() {
        let parts = RuneParts::new("❤️");
        assert_eq!(parts.len(), 6);
        assert_eq!(parts.as_str(), "❤️");
        assert_eq!(parts.as_bytes(), "❤️".as_bytes());
    }
    #[test]
    fn test_new_multiple_to_vec() {
        let parts = RuneParts::new("👌👌🏻👌🏼👌🏽👌🏾👌🏿");
        assert_eq!(parts.len(), 44);
        assert_eq!(parts.as_str(), "👌👌🏻👌🏼👌🏽👌🏾👌🏿");
        assert_eq!(parts.as_bytes(), "👌👌🏻👌🏼👌🏽👌🏾👌🏿".as_bytes());
    }

    #[test]
    fn test_rune_indexes() {
        let parts = RuneParts::new("👌👌🏻👌🏼👌🏽👌🏾👌🏿");
        assert_eq!(parts.indexes(), vec![0, 4, 12, 20, 28, 36, 44]);
    }
    #[test]
    fn test_rune_at_index() {
        let parts = RuneParts::new("👌👌🏻👌🏼👌🏽👌🏾👌🏿");
        assert_eq!(parts.rune_at_index(0), Ok(Rune::new("👌")));
        assert_eq!(parts.rune_at_index(4), Ok(Rune::new("👌🏻")));
        assert_eq!(parts.rune_at_index(12), Ok(Rune::new("👌🏼")));
        assert_eq!(parts.rune_at_index(20), Ok(Rune::new("👌🏽")));
        assert_eq!(parts.rune_at_index(28), Ok(Rune::new("👌🏾")));
        assert_eq!(parts.rune_at_index(36), Ok(Rune::new("👌🏿")));
    }
}
