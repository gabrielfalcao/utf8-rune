use std::fmt::{Debug, Display, Formatter};

use crate::{
    display_error, format_bytes,
    get_rune_cutoff_at_index, new_ptr, slice_ptr_and_length_from_display, unwrap_indent, Result, Rune, DEFAULT_INDENT,
};

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
        let input = input.to_string();
        let (ptr, length) = slice_ptr_and_length_from_display(&input);
        RuneParts { ptr, length }
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
        let ptr = new_ptr(length);
        for offset in index..cutoff {
            unsafe {
                ptr.add(offset - index)
                    .write(self.ptr.add(offset).read());
            }
        }
        Ok(Rune { ptr, length })
    }

    pub fn runes(&self) -> Vec<Rune> {
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
            let ptr = new_ptr(length);
            for offset in 0..length {
                unsafe {
                    ptr.add(offset)
                        .write(self.ptr.add(index + offset).read());
                }
            }
            runes.push(Rune { ptr, length });
            index = cutoff;
        }
        runes
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
