use std::fmt::{Debug, Formatter};

/// Represents UTF-8 byte type based on the most significant bits
/// the given byte
///
/// Examples
///
/// ```
/// use utf8_rune::ByteType;
/// let f0 = ByteType::from(0xf0u8);
/// assert_eq!(f0, ByteType::FourOrMore(0xF0));
/// assert_eq!(f0.len(), 4);
/// assert_eq!(f0.is_ascii(), false);
/// assert_eq!(f0.is_continuation(), false);
/// ```
///
/// ```
/// use utf8_rune::ByteType;
/// let e4 = ByteType::from(0xE4u8);
/// assert_eq!(e4, ByteType::Three(0xE4));
/// assert_eq!(e4.len(), 3);
/// assert_eq!(e4.is_ascii(), false);
/// assert_eq!(e4.is_continuation(), false);
/// ```
///
/// ```
/// use utf8_rune::ByteType;
/// let c3 = ByteType::from(0xC3u8);
/// assert_eq!(c3, ByteType::Two(0xC3));
/// assert_eq!(c3.len(), 2);
/// assert_eq!(c3.is_ascii(), false);
/// assert_eq!(c3.is_continuation(), false);
/// ```
///
/// ```
/// use utf8_rune::ByteType;
/// let g = ByteType::from(b'g');
/// assert_eq!(g, ByteType::Ascii(0x67));
/// assert_eq!(g.len(), 1);
/// assert_eq!(g.is_ascii(), true);
/// assert_eq!(g.is_continuation(), false);
/// ```
///
/// ```
/// use utf8_rune::ByteType;
/// let g = ByteType::from(0x80u8);
/// assert_eq!(g, ByteType::Continuation(0x80));
/// assert_eq!(g.len(), 1);
/// assert_eq!(g.is_ascii(), false);
/// assert_eq!(g.is_continuation(), true);
/// ```
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
        let indent = crate::unwrap_indent(indent);
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

#[cfg(test)]
mod test_byte_type {
    use crate::ByteType;

    #[test]
    fn test_byte_type() {
        //  "😀" => [0bf0, 0b9f, 0b98, 0b80] => [0b11110000, 0b10011111, 0b10011000, 0b10000000]
        let obf0 = ByteType::from(0b11110000u8);
        assert_eq!(obf0.len(), 4);
        let ob9f = ByteType::from(0b10011111u8);
        assert_eq!(ob9f.len(), 1);
        let ob98 = ByteType::from(0b10011000u8);
        assert_eq!(ob98.len(), 1);
        let ob80 = ByteType::from(0b10000000u8);
        assert_eq!(ob80.len(), 1);

        // "☠️" => [0be2, 0b98, 0ba0, 0bef, 0bb8, 0b8f] => [0b11100010, 0b10011000, 0b10100000, 0b11101111, 0b10111000, 0b10001111]
        let obe2 = ByteType::from(0b11100010u8);
        assert_eq!(obe2.len(), 3);
        let ob98 = ByteType::from(0b10011000u8);
        assert_eq!(ob98.len(), 1);
        let oba0 = ByteType::from(0b10100000u8);
        assert_eq!(oba0.len(), 1);
        let obef = ByteType::from(0b11101111u8);
        assert_eq!(obef.len(), 3);
        let obb8 = ByteType::from(0b10111000u8);
        assert_eq!(obb8.len(), 1);
        let ob8f = ByteType::from(0b10001111u8);
        assert_eq!(ob8f.len(), 1);
    }
}
