use std::fmt::{Debug, Display, Formatter};

use crate::{
    get_rune_cutoff_at_index,
    slice_ptr_and_length_from_display, unwrap_indent, Result,
    Rune,
};

#[derive(Clone)]
pub struct Runes {
    pub ptr: *const u8,
    pub indexes: Vec<usize>,
    pub length: usize,
}
// TODO: Deref &[&str]
impl Runes {
    pub fn new<T: Display>(input: T) -> Runes {
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
        }
    }

    pub fn len(&self) -> usize {
        self.indexes.len()
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
        self.indexes.clone()
    }

    pub fn runes(&self) -> Result<Vec<Rune>> {
        let mut runes = Vec::<Rune>::new();
        let mut indexes = self.indexes();
        if indexes.len() > 0 {
            indexes.pop();
        }
        for cutoff in indexes.into_iter() {
            runes.push(Rune::from_ptr_cutoff(self.ptr, self.length, cutoff)?);
        }
        Ok(runes)
    }
}
impl From<&str> for Runes {
    fn from(s: &str) -> Runes {
        Runes::new(s)
    }
}

impl From<String> for Runes {
    fn from(s: String) -> Runes {
        Runes::new(s)
    }
}

impl From<&String> for Runes {
    fn from(s: &String) -> Runes {
        Runes::new(s)
    }
}

impl Display for Runes {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
impl Debug for Runes {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_debug(None))
    }
}
