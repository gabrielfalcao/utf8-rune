use std::fmt::{Debug, Formatter};

use crate::internal::unwrap_indent;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ByteType {
    None,
    Ascii(u8),
    One(u8),
    Two(u8),
    Three(u8),
    FourOrMore(u8),
    Continuation(u8),
}

impl ByteType {
    pub fn new(byte: u8) -> ByteType {
        if byte < 127 {
            ByteType::Ascii(byte)
        } else {
            match byte.leading_ones() {
                0 => ByteType::One(byte),
                1 => ByteType::Continuation(byte),
                2 => ByteType::Two(byte),
                3 => ByteType::Three(byte),
                _ => ByteType::FourOrMore(byte),
            }
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            ByteType::None => "None",
            ByteType::Ascii(_) => "Ascii",
            ByteType::One(_) => "One",
            ByteType::Two(_) => "Two",
            ByteType::Three(_) => "Three",
            ByteType::FourOrMore(_) => "FourOrMore",
            ByteType::Continuation(_) => "Continuation",
        }
    }

    pub fn byte(&self) -> u8 {
        match self {
            ByteType::None => u8::default(),
            ByteType::Ascii(byte) => *byte,
            ByteType::One(byte) => *byte,
            ByteType::Two(byte) => *byte,
            ByteType::Three(byte) => *byte,
            ByteType::FourOrMore(byte) => *byte,
            ByteType::Continuation(byte) => *byte,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            ByteType::Continuation(_) | ByteType::Ascii(_) => 1,
            _ => (self.byte().leading_ones()) as usize,
        }
    }

    pub fn is_ascii(&self) -> bool {
        match self {
            ByteType::Ascii(_) => true,
            _ => false,
        }
    }

    pub fn is_continuation(&self) -> bool {
        match self {
            ByteType::Continuation(_) => true,
            _ => false,
        }
    }

    pub fn has_rune_delta(&self) -> bool {
        match self {
            ByteType::None => false,
            ByteType::Ascii(_) => false,
            ByteType::Continuation(_) => false,
            _ => true,
        }
    }

    fn as_debug(&self, indent: Option<usize>) -> String {
        let indent = unwrap_indent(indent);
        format!(
            "{}::{}{{\n{}\n}})",
            "ByteType",
            self.name(),
            [
                format!(
                    "byte: 0x{:02x},{}",
                    self.byte(),
                    if let Ok(c) = std::str::from_utf8(&[self.byte()]) {
                        format!(" // \"{c}\"")
                    } else {
                        String::new()
                    }
                ),
                format!("len: {},", self.len()),
            ]
            .iter()
            .map(|c| {
                let padding = " ".repeat(indent);
                format!("{padding}{c}")
            })
            .collect::<Vec<String>>()
            .join("\n")
        )
    }
}
impl From<u8> for ByteType {
    fn from(byte: u8) -> ByteType {
        ByteType::new(byte)
    }
}
impl From<&u8> for ByteType {
    fn from(byte: &u8) -> ByteType {
        ByteType::new(*byte)
    }
}
impl From<u16> for ByteType {
    fn from(bytes: u16) -> ByteType {
        ByteType::from(bytes.to_le_bytes()[0])
    }
}
impl From<u32> for ByteType {
    fn from(bytes: u32) -> ByteType {
        ByteType::from(bytes.to_le_bytes()[0])
    }
}
impl From<u64> for ByteType {
    fn from(bytes: u64) -> ByteType {
        ByteType::from(bytes.to_le_bytes()[0])
    }
}
impl From<usize> for ByteType {
    fn from(bytes: usize) -> ByteType {
        ByteType::from(bytes.to_le_bytes()[0])
    }
}

impl Debug for ByteType {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_debug(None))
    }
}
