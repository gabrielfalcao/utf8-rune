//! heuristic functions to find UTF-8 "[runes](crate::Rune)" within raw [u8] pointers
use crate::pointer::{
    self, get_byte_at_index, get_byte_slice_of, is_valid_utf8_str_of,
};
use crate::{ByteType, Error, Result};

/// heuristic function that determines the cutoff index at which a
/// "[rune](crate::Rune)" ends after the given index.
///
/// Example
///
/// > Note: `utf8_rune::mem` requires the `mem` feature.
///
/// ```
/// use utf8_rune::pointer::{self, get_byte_slice_of};
/// use utf8_rune::{get_rune_cutoff_at_index, Result};
///
/// let bytes = "👩🏻‍🚒👌🏿🧑🏽‍🚒👨‍🚒🌶️🎹💔🔥❤️‍🔥❤️‍🩹".as_bytes();
/// let length = bytes.len();
/// let ptr = bytes.as_ptr();
///
/// let index = 56;
/// let cutoff = get_rune_cutoff_at_index(ptr, length, index).unwrap();
/// let count = cutoff - index;
/// let slice = get_byte_slice_of(ptr, index, count);
/// assert_eq!(slice, "🎹".as_bytes());
/// ```
///
#[inline]
pub fn get_rune_cutoff_at_index<'g>(
    ptr: *const u8,
    length: usize,
    index: usize,
) -> Result<usize> {
    let ptr = pointer::copy(ptr, length)?;

    if index > length {
        return Err(Error::InvalidIndex(index, get_byte_slice_of(ptr, 0, length)));
    }
    if length == 0 {
        return Ok(index + length);
    }
    let index = index;
    let cutoff = index + 1;
    let byte = get_byte_at_index(ptr, index);
    let ty = ByteType::from(byte);
    let max = if ty.is_ascii() {
        return Ok(cutoff);
    } else if ty.is_continuation() {
        return Err(unexpected_continuation_byte_at_index_error(ptr, length, index));
    } else {
        ty.len()
    };
    let mut cutoff = index + max;
    let mut max = index + max;
    let charmax = index + 6; // +6 octets
    if cutoff <= max && max < length {
        while max < charmax && cutoff <= max && max < length {
            let next_byte = get_byte_at_index(ptr, max);
            let next_ty = ByteType::from(next_byte);
            let tcutoff = cutoff + next_ty.len();
            if let Some((count, cty)) = continuation_bytes_location(ptr, length, cutoff)
            {
                let tcutoff = cutoff + count;
                if is_valid_utf8_str_of(ptr, index, tcutoff - index) {
                    cutoff = tcutoff;
                    break;
                } else {
                    max += cty.len();
                }
            } else if next_ty.has_rune_delta() {
                if next_ty.len() < 4 {
                    if let Some((_count_, _cty_)) =
                        continuation_bytes_location(ptr, length, cutoff - index)
                    {
                        cutoff += next_ty.len();
                    }
                    break;
                } else if let Some((count, cty)) =
                    continuation_bytes_location(ptr, length, tcutoff)
                {
                    let tcutoff = cutoff + count + cty.len();
                    cutoff = tcutoff;
                    break;
                } else {
                    let delta = tcutoff - cutoff;
                    match delta {
                        4 => {
                            let mut next_chunk = [0u8; 4];
                            let next_bytes = get_byte_slice_of(
                                ptr,
                                index + delta,
                                cutoff - index + delta,
                            );
                            next_chunk.copy_from_slice(&next_bytes[0..4]);
                            match next_chunk {
                                [0xF0, 0x9F, 0x8F, 0xBB]
                                | [0xF0, 0x9F, 0x8F, 0xBC]
                                | [0xF0, 0x9F, 0x8F, 0xBD]
                                | [0xF0, 0x9F, 0x8F, 0xBE]
                                | [0xF0, 0x9F, 0x8F, 0xBF] => {
                                    cutoff += delta;
                                    break;
                                },
                                _ => {},
                            }
                        },
                        _ => {},
                    }

                    break;
                }
            } else if next_ty.is_ascii() {
                break;
            } else {
            }
            cutoff += 1;
        }
    }
    return Ok(cutoff);
}
#[inline]
pub fn split_at_first_rune<'g>(ptr: *const u8, length: usize) -> usize {
    get_rune_cutoff_at_index(ptr, length, 0).expect("should not fail at index 0")
}

#[inline]
pub fn continuation_bytes_location<'g>(
    ptr: *const u8,
    length: usize,
    index: usize,
) -> Option<(usize, ByteType)> {
    let shift: usize = (0xE2u8.leading_ones() & 0xEFu8.leading_ones()) as usize;
    if index + (shift - 1) < length {
        let zwj0 = get_byte_at_index(ptr, index);
        let zwj1 = get_byte_at_index(ptr, index + 1);
        let zwj2 = get_byte_at_index(ptr, index + 2);
        let next_rune_byte = get_byte_at_index(ptr, index + shift);
        let ty = if index + shift < length {
            ByteType::from(next_rune_byte)
        } else {
            ByteType::None
        };
        let tuple = (zwj0, zwj1, zwj2);
        match tuple {
            (0xE2, 0x80, 0x8D) => {
                let count = shift + ty.len();
                Some((count, ty))
            },
            (0xEF, 0xB8, 0x8F) => {
                let mut count = 0xEFu8.leading_ones() as usize;
                if let Some((next_count, _ty_)) =
                    continuation_bytes_location(ptr, length, index + shift)
                {
                    count += next_count
                }

                Some((count, ty))
            },
            _ => None,
        }
    } else {
        None
    }
}

#[inline]
pub fn previous_valid_cutoff<'e>(
    ptr: *const u8,
    length: usize,
    index: usize,
) -> Option<usize> {
    let ptr = pointer::copy(ptr, length).unwrap();
    if index == 0 && index == length {
        return None;
    }
    let mut previous_index = index;
    let mut byte = get_byte_at_index(ptr, previous_index);
    #[allow(unused_assignments)]
    let mut ty = ByteType::from(byte);

    while previous_index > 0 {
        ty = ByteType::from(byte);
        if !ty.has_rune_delta() {
            previous_index -= 1;
            byte = get_byte_at_index(ptr, previous_index);
        } else {
            break;
        }
    }
    if previous_index == length {
        None
    } else {
        byte = get_byte_at_index(ptr, previous_index);
        if let Some((count, cty)) =
            continuation_bytes_location(ptr, length, previous_index)
        {
            if previous_index >= cty.len() {
                if is_valid_utf8_str_of(ptr, previous_index - cty.len(), count) {
                    previous_index -= cty.len();
                    Some(previous_index)
                } else {
                    previous_index -= 1;
                    previous_valid_cutoff(ptr, length, previous_index)
                }
            } else {
                if previous_index > 0 && previous_index <= cty.len() {
                    previous_index -= 1;
                    return previous_valid_cutoff(ptr, length, previous_index);
                }

                Some(previous_index)
            }
        } else if ByteType::from(byte).has_rune_delta() {
            let cty = ByteType::from(byte);
            if is_valid_utf8_str_of(ptr, previous_index, length - previous_index) {
                return Some(previous_index);
            } else if previous_index >= cty.len() {
                previous_index -= 1;
                return previous_valid_cutoff(ptr, length, previous_index);
            } else {
                Some(previous_index)
            }
        } else if previous_index == 0 {
            None
        } else {
            Some(previous_index)
        }
    }
}

#[inline]
pub fn next_valid_cutoff<'e>(
    ptr: *const u8,
    length: usize,
    index: usize,
) -> Option<usize> {
    if length == 0 {
        return None;
    }
    if index >= length {
        return None;
    }
    let ptr = pointer::copy(ptr, length).unwrap();
    let mut next_index = index;

    while next_index < length {
        if let Some((count, _ty_)) =
            continuation_bytes_location(ptr, length, next_index)
        {
            next_index += count;
            break;
        } else if let Some((count, _ty_)) =
            continuation_bytes_location(ptr, length, next_index + 1)
        {
            next_index += count + 1;
            break;
        } else {
            let byte = get_byte_at_index(ptr, next_index);
            let ty = ByteType::from(byte);
            if ty.has_rune_delta() {
                break;
            } else if ty.is_continuation() {
                next_index += 1;
            } else {
                break;
            }
        }
    }
    if next_index == length {
        None
    } else {
        Some(next_index)
    }
}

pub(crate) fn unexpected_continuation_byte_at_index_error<'e>(
    ptr: *const u8,
    length: usize,
    index: usize,
) -> Error<'e> {
    let byte = get_byte_at_index(ptr, index);
    let previous_index = previous_valid_cutoff(ptr, length, index);
    let next_index = next_valid_cutoff(ptr, length, index);
    let slice = get_byte_slice_of(ptr, index, length);
    Error::UnexpectedContinuationByte(byte, index, previous_index, next_index, slice)
}

#[cfg(test)]
mod test_split_at_first_rune {
    use crate::pointer::{self};
    use crate::{split_at_first_rune, Result};

    #[test]
    fn test_split_at_first_rune_4_bytes() -> Result<()> {
        //  "😀" => [0xf0, 0x9f, 0x98, 0x80] => [0b11110000, 0b10011111, 0b10011000, 0b10000000]
        let (ptr, length) = pointer::from_slice("😀smiley".as_bytes())?;
        let cutoff = split_at_first_rune(ptr, length);
        assert_eq!(cutoff, 4);
        assert_eq!(length, 10);

        Ok(())
    }

    #[test]
    fn test_split_at_first_rune_6_bytes() -> Result<()> {
        // "☠️" => [0xe2, 0x98, 0xa0, 0xef, 0xb8, 0x8f] => [0b11100010, 0b10011000, 0b10100000, 0b11101111, 0b10111000, 0b10001111]
        let (ptr, length) = pointer::from_slice("☠️skull".as_bytes())?;
        let cutoff = split_at_first_rune(ptr, length);
        assert_eq!(length, 11);
        assert_eq!(cutoff, 6);

        Ok(())
    }

    #[test]
    fn test_split_at_first_ascii() -> Result<()> {
        let (ptr, length) = pointer::from_slice("abcdefghijklmnopqrstu".as_bytes())?;
        let cutoff = split_at_first_rune(ptr, length);
        assert_eq!(cutoff, 1);
        assert_eq!(length, 21);

        Ok(())
    }

    #[test]
    fn test_split_at_first_nonascii_single_byte_character() -> Result<()> {
        let (ptr, length) = pointer::from_slice("ão".as_bytes())?;
        let cutoff = split_at_first_rune(ptr, length);
        assert_eq!(cutoff, 2);
        assert_eq!(length, 3);

        Ok(())
    }

    #[test]
    fn test_split_at_first_rune_single_heart() -> Result<()> {
        let (ptr, length) = pointer::from_slice("❤️".as_bytes())?;
        let cutoff = split_at_first_rune(ptr, length);
        assert_eq!(cutoff, 6);
        assert_eq!(length, 6);
        Ok(())
    }

    #[test]
    fn test_split_at_first_rune_two_runes() -> Result<()> {
        let (ptr, length) = pointer::from_slice("❤️🦅".as_bytes())?;
        let cutoff = split_at_first_rune(ptr, length);
        assert_eq!(cutoff, 6);
        assert_eq!(length, 10);
        Ok(())
    }
}

#[cfg(test)]
mod test_get_rune_cutoff_at_index {
    use crate::pointer::{self};
    use crate::{assert_get_rune_cutoff_at_index, get_rune_cutoff_at_index, Result};
    #[test]
    fn test_get_rune_cutoff_at_first_index_single_rune() -> Result<()> {
        let (ptr, length) = pointer::from_slice("❤️".as_bytes())?;
        assert_get_rune_cutoff_at_index!(ptr, length, 6, 0, 6, "❤️");
        Ok(())
    }

    #[test]
    fn test_get_rune_cutoff_empty() -> Result<()> {
        let (ptr, length) = pointer::from_slice("".as_bytes())?;
        assert_get_rune_cutoff_at_index!(ptr, length, 0, 0, 0, "");
        Ok(())
    }

    #[test]
    fn test_get_rune_cutoff_at_various_indexes_4_bytes() -> Result<()> {
        //  "😀" => [0xf0, 0x9f, 0x98, 0x80] => [0b11110000, 0b10011111, 0b10011000, 0b10000000]
        let (ptr, length) = pointer::from_slice("smiley😀smiley".as_bytes())?;
        assert_get_rune_cutoff_at_index!(ptr, length, 16, 6, 10, "😀");

        Ok(())
    }

    #[test]
    fn test_get_rune_cutoff_at_various_indexes_6_bytes() -> Result<()> {
        // "☠️" => [0xe2, 0x98, 0xa0, 0xef, 0xb8, 0x8f] => [0b11100010, 0b10011000, 0b10100000, 0b11101111, 0b10111000, 0b10001111]
        let (ptr, length) = pointer::from_slice("skull☠️skull".as_bytes())?;
        assert_get_rune_cutoff_at_index!(ptr, length, 16, 5, 11, "☠️");

        Ok(())
    }

    #[test]
    fn test_get_rune_cutoff_at_various_indexes_ascii() -> Result<()> {
        let (ptr, length) = pointer::from_slice("abcdefghijklmnopqrstu".as_bytes())?;
        assert_get_rune_cutoff_at_index!(ptr, length, 21, 7, 8, "h");

        Ok(())
    }

    #[test]
    fn test_get_rune_cutoff_at_various_indexes_non_ascii() -> Result<()> {
        // "🦅" => length=4 => [0xf0, 0x9f, 0xa6, 0x85] => [0b11110000, 0b10011111, 0b10100110, 0b10000101] => [240, 159, 166, 133]
        // "ã" => length=2 => [0xc3, 0xa3] => [0b11000011, 0b10100011] => [195, 163]

        let (ptr, length) = pointer::from_slice("falcão🦅".as_bytes())?;
        assert_get_rune_cutoff_at_index!(ptr, length, 11, 4, 6, "ã");
        assert_get_rune_cutoff_at_index!(ptr, length, 11, 6, 7, "o");
        assert_get_rune_cutoff_at_index!(ptr, length, 11, 7, 11, "🦅");
        Ok(())
    }

    #[test]
    fn test_get_rune_cutoff_at_first_index() -> Result<()> {
        let (ptr, length) = pointer::from_slice("❤️🦅".as_bytes())?;
        assert_get_rune_cutoff_at_index!(ptr, length, 10, 0, 6, "❤️");
        assert_get_rune_cutoff_at_index!(ptr, length, 10, 6, 10, "🦅");
        Ok(())
    }

    #[test]
    fn test_get_rune_cutoff_unexpected_continuation_byte() -> Result<()> {
        let (ptr, _) = pointer::from_slice("❤️🦅".as_bytes())?;
        let cutoff = get_rune_cutoff_at_index(ptr, 10, 4);

        assert!(cutoff.is_err());
        let err = cutoff.err().unwrap();
        assert_eq!(err.previous_valid_cutoff(), Some(0));
        assert_eq!(err.next_valid_cutoff(), Some(6));
        Ok(())
    }

    #[test]
    fn test_get_rune_cutoff_at_various_indexes_94_bytes() -> Result<()> {
        // "👩🏻‍🚒" => length=15 => [0xf0, 0x9f, 0x91, 0xa9, 0xf0, 0x9f, 0x8f, 0xbb, 0xe2, 0x80, 0x8d, 0xf0, 0x9f, 0x9a, 0x92] =>
        // [0b11110000, 0b10011111, 0b10010001, 0b10101001, 0b11110000, 0b10011111, 0b10001111, 0b10111011,
        //  0b11100010, 0b10000000, 0b10001101, 0b11110000, 0b10011111, 0b10011010, 0b10010010] => [240, 159, 145, 169, 240, 159, 143, 187, 226, 128, 141, 240, 159, 154, 146]
        // "👌🏿" => length=8 => [0xf0, 0x9f, 0x91, 0x8c, 0xf0, 0x9f, 0x8f, 0xbf] =>
        // [0b11110000, 0b10011111, 0b10010001, 0b10001100, 0b11110000, 0b10011111, 0b10001111, 0b10111111] => [240, 159, 145, 140, 240, 159, 143, 191]
        // "🧑🏽‍🚒" => length=15 => [0xf0, 0x9f, 0xa7, 0x91, 0xf0, 0x9f, 0x8f, 0xbd, 0xe2, 0x80, 0x8d, 0xf0, 0x9f, 0x9a, 0x92] => [0b11110000, 0b10011111, 0b10100111, 0b10010001, 0b11110000, 0b10011111, 0b10001111, 0b10111101, 0b11100010, 0b10000000, 0b10001101, 0b11110000, 0b10011111, 0b10011010, 0b10010010] => [240, 159, 167, 145, 240, 159, 143, 189, 226, 128, 141, 240, 159, 154, 146]
        // "👨‍🚒" => length=11 => [0xf0, 0x9f, 0x91, 0xa8, 0xe2, 0x80, 0x8d, 0xf0, 0x9f, 0x9a, 0x92] => [0b11110000, 0b10011111, 0b10010001, 0b10101000, 0b11100010, 0b10000000, 0b10001101, 0b11110000, 0b10011111, 0b10011010, 0b10010010] => [240, 159, 145, 168, 226, 128, 141, 240, 159, 154, 146]
        // "🌶️" => length=7 => [0xf0, 0x9f, 0x8c, 0xb6, 0xef, 0xb8, 0x8f] =>
        // [0b11110000, 0b10011111, 0b10001100, 0b10110110, 0b11101111, 0b10111000, 0b10001111] => [240, 159, 140, 182, 239, 184, 143]
        // "🎹" => length=4 => [0xf0, 0x9f, 0x8e, 0xb9] => [0b11110000, 0b10011111, 0b10001110, 0b10111001] => [240, 159, 142, 185]
        // "💔" => length=4 => [0xf0, 0x9f, 0x92, 0x94] => [0b11110000, 0b10011111, 0b10010010, 0b10010100] => [240, 159, 146, 148]
        // "🔥" => length=4 => [0xf0, 0x9f, 0x94, 0xa5] => [0b11110000, 0b10011111, 0b10010100, 0b10100101] => [240, 159, 148, 165]
        // "❤️‍🔥" => length=13 => [0xe2, 0x9d, 0xa4, 0xef, 0xb8, 0x8f, 0xe2, 0x80, 0x8d, 0xf0, 0x9f, 0x94, 0xa5] => [0b11100010, 0b10011101, 0b10100100, 0b11101111, 0b10111000, 0b10001111, 0b11100010, 0b10000000, 0b10001101, 0b11110000, 0b10011111, 0b10010100, 0b10100101] => [226, 157, 164, 239, 184, 143, 226, 128, 141, 240, 159, 148, 165]
        // "❤️‍🩹" => length=13 => [0xe2, 0x9d, 0xa4, 0xef, 0xb8, 0x8f, 0xe2, 0x80, 0x8d, 0xf0, 0x9f, 0xa9, 0xb9] => [0b11100010, 0b10011101, 0b10100100, 0b11101111, 0b10111000, 0b10001111, 0b11100010, 0b10000000, 0b10001101, 0b11110000, 0b10011111, 0b10101001, 0b10111001] => [226, 157, 164, 239, 184, 143, 226, 128, 141, 240, 159, 169, 185]
        let (ptr, length) = pointer::from_slice("👩🏻‍🚒👌🏿🧑🏽‍🚒👨‍🚒🌶️🎹💔🔥❤️‍🔥❤️‍🩹".as_bytes())?;
        assert_get_rune_cutoff_at_index!(ptr, length, 94, 0, 15, "👩🏻‍🚒");
        assert_get_rune_cutoff_at_index!(ptr, length, 94, 15, 23, "👌🏿");
        assert_get_rune_cutoff_at_index!(ptr, length, 94, 23, 38, "🧑🏽‍🚒");
        assert_get_rune_cutoff_at_index!(ptr, length, 94, 38, 49, "👨‍🚒");
        assert_get_rune_cutoff_at_index!(ptr, length, 94, 49, 56, "🌶️");
        assert_get_rune_cutoff_at_index!(ptr, length, 94, 56, 60, "🎹");
        assert_get_rune_cutoff_at_index!(ptr, length, 94, 60, 64, "💔");
        assert_get_rune_cutoff_at_index!(ptr, length, 94, 64, 68, "🔥");
        assert_get_rune_cutoff_at_index!(ptr, length, 94, 68, 81, "❤️‍🔥");
        assert_get_rune_cutoff_at_index!(ptr, length, 94, 81, 94, "❤️‍🩹");

        Ok(())
    }
    #[macro_export]
    macro_rules! assert_get_rune_cutoff_at_index {
    (
        $ptr:expr,
        $length:expr,
        $expected_length:literal,
        $index:literal,
        $cutoff:literal,
        $expected:literal
        $(,)?
    ) => {{
        use debug_et_diagnostics::{ansi, fore, from_bytes, indent};
        use crate::{get_byte_slice_of, format_bytes, RuneParts};

        // debug_et_diagnostics::step!(fg=line, format!("expecting {} from index..cutoff {}..{}", $expected, $index, $cutoff));

        let slice = get_byte_slice_of($ptr, 0, $length)
            .iter()
            .map(Clone::clone)
            .map(|c| fore(c.to_string(), from_bytes(&[c]).into()))
            .collect::<Vec<String>>()
            .join(", ");
        let cutoff = get_rune_cutoff_at_index($ptr, $length, $index)?;
        let count = $cutoff - $index;
        assert_eq!(
            $length,
            $expected_length,
            "{}",
            [
                fore("expected length to be", 231),
                fore(format!("{}", $expected_length), 196),
            ]
            .join(" ")
        );
        assert_eq!(
            cutoff,
            $cutoff,
            "{}",
            [
                fore("expected cutoff", 231),
                fore(format!("{}", cutoff), 220),
                fore("to be", 231),
                fore(format!("{}", $cutoff), 196),
                fore("so as to match rune:", 231),
                ansi(format!("{}", $expected), 16, 231),
                format_expected_rune($expected),
                fore("instead of:", 231),
                ansi(format!("{}", {
                    let slice = get_byte_slice_of($ptr, $index, cutoff);
                    let string = std::str::from_utf8(slice)
                        .map(|c| ansi(format!("{c}"), 16, 231))
                        .unwrap_or_else(|e| {
                            let slice = get_byte_slice_of($ptr, $index, e.valid_up_to());
                            std::str::from_utf8(slice).map(String::from).unwrap_or_default()
                        });
                    string
                }), 16, 231),
                {
                    format!(
                        "\n{}\n",
                        [
                            String::new(),
                            fore(".i.e.:", 231),
                            fore(
                                indent!(format!(
                                    "get_byte_slice_of(ptr, {}, {})",
                                    $index, $cutoff
                                )),
                                231,
                            ),
                            fore("is:", 231),
                            format_bytes(get_byte_slice_of($ptr, $index, count), None),
                        ]
                        .iter()
                        .map(|c| format!("        {c}"))
                        .collect::<Vec<String>>()
                        .join("\n")
                    )
                }
            ]
            .join(" ")
        );

        let index = $index;
        let length = $length - index;
        let actual = match RuneParts::from_raw_parts($ptr, $length)
            .rune_at_index($index) {
                Ok(actual) => actual.as_str().to_string(),
                Err(error) => {
                    panic!("{}:{} RuneParts::from_raw_parts({:#?}, {})\n{error}", file!(), line!(), $ptr, $length);
                }
            };

        let expected = $expected.to_string();
        assert_eq!(
            actual,
            expected.to_string(),
            "{}",
            ansi(
                [
                    String::new(),
                    [
                        fore("expected", 82),
                        fore("rune as string", 231),
                        fore(format!("{expected}"), 82),
                    ]
                    .join(" "),
                    [
                        fore("actual", 196),
                        fore("rune as string", 231),
                        fore(format!("{actual}"), 196),
                    ]
                    .join(" "),
                    [
                        fore("from slice ", 220),
                        fore("[", 231),
                        format!("{slice}"),
                        fore("]", 231),
                        [
                            String::new(),
                            fore(format!("index={index}"), 82),
                            fore(format!("cutoff={cutoff}"), 202),
                            fore(format!("length={length}"), 74),
                        ]
                        .join(" "),
                    ]
                    .join(""),
                    fore(format!("expected cutoff={}", $cutoff), 196),
                ]
                .join("\n"),
                231,
                16
            ),
        );
    }};
}

    fn format_expected_rune(c: &str) -> String {
        use debug_et_diagnostics::color::byte_hex;
        format!(
            "\"{c}\" => [{}]",
            c.as_bytes()
                .iter()
                .map(Clone::clone)
                .map(byte_hex)
                .collect::<Vec<String>>()
                .join(", "),
        )
    }
}

#[cfg(test)]
mod test_continuation_bytes_location {
    use crate::pointer::{self};
    use crate::{continuation_bytes_location, Result};
    #[test]
    fn test_continuation_byte_0x200d() -> Result<()> {
        let (ptr, length) = pointer::from_display("👩🏻‍🚒")?;

        assert_eq!(
            "👩🏻‍🚒".as_bytes().to_vec(),
            vec![
                0xF0, 0x9F, 0x91, 0xA9, 0xF0, 0x9F, 0x8F, 0xBB, 0xE2, 0x80, 0x8D, 0xF0,
                0x9F, 0x9A, 0x92
            ]
        );
        let location = continuation_bytes_location(ptr, length, 8);

        assert_eq!(location.is_some(), true);
        let (count, ty) = location.unwrap();
        assert_eq!(ty.len(), 4);
        assert_eq!(ty.byte(), 0xF0);
        assert_eq!(count, 7);

        Ok(())
    }
    #[test]
    fn test_continuation_byte_0xfe0f() -> Result<()> {
        let (ptr, length) = pointer::from_display("❤️‍🔥")?;
        assert_eq!(
            "❤️‍🔥".as_bytes().to_vec(),
            vec![
                0xE2, 0x9D, 0xA4, 0xEF, 0xB8, 0x8F, 0xE2, 0x80, 0x8D, 0xF0, 0x9F, 0x94,
                0xA5
            ]
        );
        let location = continuation_bytes_location(ptr, length, 3);

        assert_eq!(location.is_some(), true);
        let (count, ty) = location.unwrap();
        assert_eq!(ty.len(), 3);
        assert_eq!(ty.byte(), 0xE2);
        assert_eq!(count, 10);

        Ok(())
    }
}

#[cfg(test)]
mod test_next_valid_cutoff {

    use crate::pointer::{self};
    use crate::{
        assert_none_next_valid_cutoff, assert_some_next_valid_cutoff,
        next_valid_cutoff, Result, RuneParts,
    };
    #[test]
    fn test_next_valid_cutoff_parts() -> Result<()> {
        let (ptr, length) = pointer::from_slice("👌👌🏻👌🏼👌🏽👌🏾👌🏿".as_bytes())?;
        assert_some_next_valid_cutoff!(ptr, length, 44, 0, 0, "👌");
        assert_some_next_valid_cutoff!(ptr, length, 44, 1, 4, "👌🏻");
        assert_some_next_valid_cutoff!(ptr, length, 44, 2, 4, "👌🏻");
        assert_some_next_valid_cutoff!(ptr, length, 44, 3, 4, "👌🏻");
        assert_some_next_valid_cutoff!(ptr, length, 44, 4, 4, "👌🏻");

        assert_some_next_valid_cutoff!(ptr, length, 44, 5, 8, "🏻");
        assert_some_next_valid_cutoff!(ptr, length, 44, 6, 8, "🏻");
        assert_some_next_valid_cutoff!(ptr, length, 44, 7, 8, "🏻");
        assert_some_next_valid_cutoff!(ptr, length, 44, 8, 8, "🏻");
        assert_some_next_valid_cutoff!(ptr, length, 44, 9, 12, "👌🏼");
        assert_some_next_valid_cutoff!(ptr, length, 44, 10, 12, "👌🏼");
        assert_some_next_valid_cutoff!(ptr, length, 44, 11, 12, "👌🏼");
        assert_some_next_valid_cutoff!(ptr, length, 44, 12, 12, "👌🏼");

        assert_some_next_valid_cutoff!(ptr, length, 44, 13, 16, "🏼");
        assert_some_next_valid_cutoff!(ptr, length, 44, 14, 16, "🏼");
        assert_some_next_valid_cutoff!(ptr, length, 44, 15, 16, "🏼");
        assert_some_next_valid_cutoff!(ptr, length, 44, 16, 16, "🏼");
        assert_some_next_valid_cutoff!(ptr, length, 44, 17, 20, "👌🏽");
        assert_some_next_valid_cutoff!(ptr, length, 44, 18, 20, "👌🏽");
        assert_some_next_valid_cutoff!(ptr, length, 44, 19, 20, "👌🏽");
        assert_some_next_valid_cutoff!(ptr, length, 44, 20, 20, "👌🏽");

        assert_some_next_valid_cutoff!(ptr, length, 44, 21, 24, "🏽");
        assert_some_next_valid_cutoff!(ptr, length, 44, 22, 24, "🏽");
        assert_some_next_valid_cutoff!(ptr, length, 44, 23, 24, "🏽");
        assert_some_next_valid_cutoff!(ptr, length, 44, 24, 24, "🏽");
        assert_some_next_valid_cutoff!(ptr, length, 44, 25, 28, "👌🏾");
        assert_some_next_valid_cutoff!(ptr, length, 44, 26, 28, "👌🏾");
        assert_some_next_valid_cutoff!(ptr, length, 44, 27, 28, "👌🏾");
        assert_some_next_valid_cutoff!(ptr, length, 44, 28, 28, "👌🏾");

        assert_some_next_valid_cutoff!(ptr, length, 44, 29, 32, "🏾");
        assert_some_next_valid_cutoff!(ptr, length, 44, 30, 32, "🏾");
        assert_some_next_valid_cutoff!(ptr, length, 44, 31, 32, "🏾");
        assert_some_next_valid_cutoff!(ptr, length, 44, 32, 32, "🏾");
        assert_some_next_valid_cutoff!(ptr, length, 44, 33, 36, "👌🏿");
        assert_some_next_valid_cutoff!(ptr, length, 44, 34, 36, "👌🏿");
        assert_some_next_valid_cutoff!(ptr, length, 44, 35, 36, "👌🏿");
        assert_some_next_valid_cutoff!(ptr, length, 44, 36, 36, "👌🏿");

        assert_some_next_valid_cutoff!(ptr, length, 44, 37, 40, "🏿");
        assert_some_next_valid_cutoff!(ptr, length, 44, 38, 40, "🏿");
        assert_some_next_valid_cutoff!(ptr, length, 44, 39, 40, "🏿");
        assert_some_next_valid_cutoff!(ptr, length, 44, 40, 40, "🏿");
        assert_none_next_valid_cutoff!(ptr, length, 44, 41);
        assert_none_next_valid_cutoff!(ptr, length, 44, 42);
        assert_none_next_valid_cutoff!(ptr, length, 44, 43);
        assert_none_next_valid_cutoff!(ptr, length, 44, 44);
        assert_none_next_valid_cutoff!(ptr, length, 44, 45);
        Ok(())
    }

    #[test]
    fn test_next_valid_cutoff_at_first_index_single_rune() -> Result<()> {
        let (ptr, length) = pointer::from_slice("❤️".as_bytes())?;
        assert_some_next_valid_cutoff!(ptr, length, 6, 0, 0, "❤️");
        Ok(())
    }

    #[test]
    fn test_next_valid_cutoff_empty() -> Result<()> {
        let (ptr, length) = pointer::from_slice("".as_bytes())?;
        assert_none_next_valid_cutoff!(ptr, length, 0, 0);

        Ok(())
    }

    #[test]
    fn test_next_valid_cutoff_at_various_indexes_6_bytes() -> Result<()> {
        let (ptr, length) = pointer::from_slice("skull☠️skull".as_bytes())?;
        assert_some_next_valid_cutoff!(ptr, length, 16, 0, 0, "s");
        assert_some_next_valid_cutoff!(ptr, length, 16, 4, 4, "l");
        assert_some_next_valid_cutoff!(ptr, length, 16, 5, 5, "☠️");
        Ok(())
    }

    #[test]
    fn test_next_valid_cutoff_at_various_indexes_4_bytes() -> Result<()> {
        let (ptr, length) = pointer::from_slice("smiley😀smiley".as_bytes())?;
        assert_some_next_valid_cutoff!(ptr, length, 16, 5, 5, "y");
        assert_some_next_valid_cutoff!(ptr, length, 16, 6, 6, "😀");
        Ok(())
    }

    #[test]
    fn test_next_valid_cutoff_at_various_indexes_ascii() -> Result<()> {
        let (ptr, length) = pointer::from_slice("abcdefghijklmnopqrstu".as_bytes())?;
        assert_some_next_valid_cutoff!(ptr, length, 21, 7, 7, "h");

        Ok(())
    }

    #[test]
    fn test_next_valid_cutoff_at_various_indexes_non_ascii() -> Result<()> {
        let (ptr, length) = pointer::from_slice("falcão🦅".as_bytes())?;
        assert_some_next_valid_cutoff!(ptr, length, 11, 4, 4, "ã");
        assert_some_next_valid_cutoff!(ptr, length, 11, 5, 6, "o");
        assert_some_next_valid_cutoff!(ptr, length, 11, 6, 6, "o");
        assert_some_next_valid_cutoff!(ptr, length, 11, 7, 7, "🦅");
        assert_none_next_valid_cutoff!(ptr, length, 11, 8);
        Ok(())
    }

    #[test]
    fn test_next_valid_cutoff_at_first_index() -> Result<()> {
        let (ptr, length) = pointer::from_slice("❤️🦅".as_bytes())?;
        assert_some_next_valid_cutoff!(ptr, length, 10, 0, 0, "❤️");
        assert_some_next_valid_cutoff!(ptr, length, 10, 1, 6, "🦅");
        assert_some_next_valid_cutoff!(ptr, length, 10, 2, 6, "🦅");
        assert_some_next_valid_cutoff!(ptr, length, 10, 3, 6, "🦅");
        assert_some_next_valid_cutoff!(ptr, length, 10, 4, 6, "🦅");
        assert_some_next_valid_cutoff!(ptr, length, 10, 5, 6, "🦅");
        assert_some_next_valid_cutoff!(ptr, length, 10, 6, 6, "🦅");
        assert_none_next_valid_cutoff!(ptr, length, 10, 7);

        Ok(())
    }

    #[test]
    fn test_next_valid_cutoff_at_various_indexes_94_bytes() -> Result<()> {
        let (ptr, length) = pointer::from_slice("👩🏻‍🚒👌🏿🧑🏽‍🚒👨‍🚒🌶️🎹💔🔥❤️‍🔥❤️‍🩹".as_bytes())?;
        assert_some_next_valid_cutoff!(ptr, length, 94, 0, 0, "👩🏻‍🚒");
        assert_some_next_valid_cutoff!(ptr, length, 94, 1, 4, "🏻\u{200d}🚒");
        assert_some_next_valid_cutoff!(ptr, length, 94, 2, 4, "🏻\u{200d}🚒");
        assert_some_next_valid_cutoff!(ptr, length, 94, 3, 4, "🏻\u{200d}🚒");
        assert_some_next_valid_cutoff!(ptr, length, 94, 4, 4, "🏻\u{200d}🚒");
        assert_some_next_valid_cutoff!(ptr, length, 94, 5, 15, "👌🏿");
        assert_some_next_valid_cutoff!(ptr, length, 94, 15, 15, "👌🏿");
        assert_some_next_valid_cutoff!(ptr, length, 94, 16, 19, "🏿");
        assert_some_next_valid_cutoff!(ptr, length, 94, 17, 19, "🏿");
        assert_some_next_valid_cutoff!(ptr, length, 94, 18, 19, "🏿");
        assert_some_next_valid_cutoff!(ptr, length, 94, 19, 19, "🏿");
        assert_some_next_valid_cutoff!(ptr, length, 94, 20, 23, "🧑🏽‍🚒");
        assert_some_next_valid_cutoff!(ptr, length, 94, 21, 23, "🧑🏽‍🚒");
        assert_some_next_valid_cutoff!(ptr, length, 94, 22, 23, "🧑🏽‍🚒");
        assert_some_next_valid_cutoff!(ptr, length, 94, 23, 23, "🧑🏽‍🚒");

        assert_some_next_valid_cutoff!(ptr, length, 94, 24, 27, "🏽\u{200d}🚒");
        assert_some_next_valid_cutoff!(ptr, length, 94, 25, 27, "🏽\u{200d}🚒");
        assert_some_next_valid_cutoff!(ptr, length, 94, 26, 27, "🏽\u{200d}🚒");
        assert_some_next_valid_cutoff!(ptr, length, 94, 27, 27, "🏽\u{200d}🚒");
        assert_some_next_valid_cutoff!(ptr, length, 94, 28, 38, "👨\u{200d}🚒");
        assert_some_next_valid_cutoff!(ptr, length, 94, 29, 38, "👨\u{200d}🚒");
        assert_some_next_valid_cutoff!(ptr, length, 94, 30, 38, "👨\u{200d}🚒");
        assert_some_next_valid_cutoff!(ptr, length, 94, 31, 38, "👨\u{200d}🚒");
        assert_some_next_valid_cutoff!(ptr, length, 94, 32, 34, "🚒👨\u{200d}🚒");
        assert_some_next_valid_cutoff!(ptr, length, 94, 33, 34, "🚒👨\u{200d}🚒");
        assert_some_next_valid_cutoff!(ptr, length, 94, 34, 34, "🚒👨\u{200d}🚒");
        assert_some_next_valid_cutoff!(ptr, length, 94, 35, 38, "👨\u{200d}🚒");
        assert_some_next_valid_cutoff!(ptr, length, 94, 36, 38, "👨\u{200d}🚒");
        assert_some_next_valid_cutoff!(ptr, length, 94, 37, 38, "👨\u{200d}🚒");
        assert_some_next_valid_cutoff!(ptr, length, 94, 38, 38, "👨‍🚒");
        assert_some_next_valid_cutoff!(ptr, length, 94, 39, 49, "🌶️");
        assert_some_next_valid_cutoff!(ptr, length, 94, 40, 49, "🌶️");
        assert_some_next_valid_cutoff!(ptr, length, 94, 41, 49, "🌶️");
        assert_some_next_valid_cutoff!(ptr, length, 94, 42, 49, "🌶️");
        assert_some_next_valid_cutoff!(ptr, length, 94, 43, 45, "🚒🌶️");
        assert_some_next_valid_cutoff!(ptr, length, 94, 44, 45, "🚒🌶️");
        assert_some_next_valid_cutoff!(ptr, length, 94, 45, 45, "🚒🌶️");
        assert_some_next_valid_cutoff!(ptr, length, 94, 46, 49, "🌶️");
        assert_some_next_valid_cutoff!(ptr, length, 94, 47, 49, "🌶️");
        assert_some_next_valid_cutoff!(ptr, length, 94, 48, 49, "🌶️");
        assert_some_next_valid_cutoff!(ptr, length, 94, 49, 49, "🌶️");
        assert_some_next_valid_cutoff!(ptr, length, 94, 49, 49, "🌶️");
        assert_some_next_valid_cutoff!(ptr, length, 94, 50, 56, "🎹");
        assert_some_next_valid_cutoff!(ptr, length, 94, 51, 56, "🎹");
        assert_some_next_valid_cutoff!(ptr, length, 94, 52, 56, "🎹");
        assert_some_next_valid_cutoff!(ptr, length, 94, 53, 56, "🎹");
        assert_some_next_valid_cutoff!(ptr, length, 94, 54, 56, "🎹");
        assert_some_next_valid_cutoff!(ptr, length, 94, 55, 56, "🎹");
        assert_some_next_valid_cutoff!(ptr, length, 94, 56, 56, "🎹");
        assert_some_next_valid_cutoff!(ptr, length, 94, 57, 60, "💔");
        assert_some_next_valid_cutoff!(ptr, length, 94, 58, 60, "💔");
        assert_some_next_valid_cutoff!(ptr, length, 94, 59, 60, "💔");
        assert_some_next_valid_cutoff!(ptr, length, 94, 60, 60, "💔");
        assert_some_next_valid_cutoff!(ptr, length, 94, 61, 64, "🔥");
        assert_some_next_valid_cutoff!(ptr, length, 94, 62, 64, "🔥");
        assert_some_next_valid_cutoff!(ptr, length, 94, 63, 64, "🔥");
        assert_some_next_valid_cutoff!(ptr, length, 94, 64, 64, "🔥");
        assert_some_next_valid_cutoff!(ptr, length, 94, 65, 68, "❤️‍🔥");
        assert_some_next_valid_cutoff!(ptr, length, 94, 66, 68, "❤️‍🔥");
        assert_some_next_valid_cutoff!(ptr, length, 94, 67, 68, "❤️‍🔥");
        assert_some_next_valid_cutoff!(ptr, length, 94, 68, 68, "❤️‍🔥");
        assert_some_next_valid_cutoff!(ptr, length, 94, 69, 81, "❤️‍🩹");
        assert_some_next_valid_cutoff!(ptr, length, 94, 70, 81, "❤️‍🩹");
        assert_some_next_valid_cutoff!(ptr, length, 94, 71, 81, "❤️‍🩹");
        assert_some_next_valid_cutoff!(ptr, length, 94, 72, 81, "❤️‍🩹");
        assert_some_next_valid_cutoff!(ptr, length, 94, 73, 81, "❤️‍🩹");
        assert_some_next_valid_cutoff!(ptr, length, 94, 74, 81, "❤️‍🩹");
        assert_some_next_valid_cutoff!(ptr, length, 94, 75, 77, "🔥");
        assert_some_next_valid_cutoff!(ptr, length, 94, 76, 77, "🔥");
        assert_some_next_valid_cutoff!(ptr, length, 94, 77, 77, "🔥");
        assert_some_next_valid_cutoff!(ptr, length, 94, 78, 81, "❤️‍🩹");
        assert_some_next_valid_cutoff!(ptr, length, 94, 79, 81, "❤️‍🩹");
        assert_some_next_valid_cutoff!(ptr, length, 94, 80, 81, "❤️‍🩹");
        assert_some_next_valid_cutoff!(ptr, length, 94, 81, 81, "❤️‍🩹");
        assert_none_next_valid_cutoff!(ptr, length, 94, 82);
        assert_none_next_valid_cutoff!(ptr, length, 94, 83);
        assert_none_next_valid_cutoff!(ptr, length, 94, 84);
        assert_none_next_valid_cutoff!(ptr, length, 94, 85);
        assert_none_next_valid_cutoff!(ptr, length, 94, 86);
        assert_none_next_valid_cutoff!(ptr, length, 94, 87);
        assert_some_next_valid_cutoff!(ptr, length, 94, 88, 90, "🩹");
        assert_some_next_valid_cutoff!(ptr, length, 94, 89, 90, "🩹");
        assert_some_next_valid_cutoff!(ptr, length, 94, 90, 90, "🩹");
        assert_none_next_valid_cutoff!(ptr, length, 94, 91);
        assert_none_next_valid_cutoff!(ptr, length, 94, 92);
        assert_none_next_valid_cutoff!(ptr, length, 94, 93);
        assert_none_next_valid_cutoff!(ptr, length, 94, 94);

        Ok(())
    }

    #[macro_export]
    macro_rules! assert_some_next_valid_cutoff {
    (
        $ptr:expr,
        $length:expr,
        $expected_length:literal,
        $invalid_index:literal,
        $expected_valid_index:literal,
        $expected_rune_str:literal
        $(,)?
    ) => {{
        // debug_et_diagnostics::step!(fg=line, format!("expecting next_valid_cutoff from invalid index {} to be {} matching rune \"{}\"", $invalid_index, $expected_valid_index, $expected_rune_str));

        assert_eq!($length, $expected_length, "expected length to be {} rather than {}", $expected_length, $length);
        let result = next_valid_cutoff($ptr, $length, $invalid_index);
        assert!(result.is_some(), "expected next_valid_cutoff at {} to not be None", $invalid_index);
        let actual = result.unwrap();
        assert_eq!(actual, $expected_valid_index, "expected next_valid_cutoff to be {} rather than {}", $expected_valid_index, actual);
        let parts = RuneParts::from_raw_parts($ptr, $length);
        let result = parts.rune_at_index(actual);
        assert!(result.is_ok(), "expected valid Rune at index to be {} but got error: {}", $expected_valid_index, result.err().map(|err|err.to_string()).unwrap_or_default());
        let rune = result.unwrap();
        assert_eq!(rune.as_str(), $expected_rune_str,
                   "expected rune at index {} to match \"{}\" rather than \"{}\"",
                   actual, $expected_rune_str, rune.as_str());
    }};
}

    #[macro_export]
    macro_rules! assert_none_next_valid_cutoff {
        (
            $ptr:expr,
            $length:expr,
            $expected_length:literal,
            $invalid_index:literal $(,)?
        ) => {{
            // debug_et_diagnostics::step!(fg = (line!() as u8),format!("expecting next_valid_cutoff from invalid index {} to be None",$invalid_index));
            assert_eq!(
                $length, $expected_length,
                "expected length to be {} rather than {}",
                $expected_length, $length
            );
            let result = next_valid_cutoff($ptr, $length, $invalid_index);
            assert!(
                result.is_none(),
                "expected next_valid_cutoff at {} to not be None but is actually {:#?}",
                $invalid_index,
                result
            );
        }};
    }
}

#[cfg(test)]
mod test_previous_valid_cutoff {
    use crate::pointer::{self};
    use crate::{
        assert_none_previous_valid_cutoff, assert_some_previous_valid_cutoff,
        previous_valid_cutoff, Result, RuneParts,
    };

    #[test]
    fn test_previous_valid_cutoff_parts() -> Result<()> {
        let (ptr, length) = pointer::from_slice("👌👌🏻👌🏼👌🏽👌🏾👌🏿".as_bytes())?;
        assert_some_previous_valid_cutoff!(ptr, length, 44, 0, 0, "👌");
        assert_some_previous_valid_cutoff!(ptr, length, 44, 1, 0, "👌");
        assert_some_previous_valid_cutoff!(ptr, length, 44, 5, 4, "👌🏻");
        assert_some_previous_valid_cutoff!(ptr, length, 44, 13, 12, "👌🏼");
        assert_some_previous_valid_cutoff!(ptr, length, 44, 21, 20, "👌🏽");
        assert_some_previous_valid_cutoff!(ptr, length, 44, 29, 28, "👌🏾");
        assert_some_previous_valid_cutoff!(ptr, length, 44, 37, 36, "👌🏿");
        Ok(())
    }
    #[test]
    fn test_previous_valid_cutoff_at_first_index_single_rune() -> Result<()> {
        let (ptr, length) = pointer::from_slice("❤️".as_bytes())?;
        assert_some_previous_valid_cutoff!(ptr, length, 6, 0, 0, "❤️");
        Ok(())
    }

    #[test]
    fn test_previous_valid_cutoff_empty() -> Result<()> {
        let (ptr, length) = pointer::from_slice("".as_bytes())?;
        assert_none_previous_valid_cutoff!(ptr, length, 0, 0);

        Ok(())
    }

    #[test]
    fn test_previous_valid_cutoff_at_various_indexes_6_bytes() -> Result<()> {
        let (ptr, length) = pointer::from_slice("skull☠️skull".as_bytes())?;
        assert_none_previous_valid_cutoff!(ptr, length, 16, 0);
        assert_none_previous_valid_cutoff!(ptr, length, 16, 1);
        assert_none_previous_valid_cutoff!(ptr, length, 16, 2);
        assert_none_previous_valid_cutoff!(ptr, length, 16, 3);
        assert_none_previous_valid_cutoff!(ptr, length, 16, 4);
        assert_some_previous_valid_cutoff!(ptr, length, 16, 5, 5, "☠️");
        assert_some_previous_valid_cutoff!(ptr, length, 16, 6, 5, "☠️");
        assert_some_previous_valid_cutoff!(ptr, length, 16, 7, 5, "☠️");
        assert_some_previous_valid_cutoff!(ptr, length, 16, 8, 5, "☠️");
        assert_some_previous_valid_cutoff!(ptr, length, 16, 9, 5, "☠️");
        assert_some_previous_valid_cutoff!(ptr, length, 16, 10, 5, "☠️");
        assert_some_previous_valid_cutoff!(ptr, length, 16, 11, 5, "☠️");
        assert_some_previous_valid_cutoff!(ptr, length, 16, 12, 5, "☠️");
        assert_some_previous_valid_cutoff!(ptr, length, 16, 13, 5, "☠️");
        assert_some_previous_valid_cutoff!(ptr, length, 16, 14, 5, "☠️");
        assert_some_previous_valid_cutoff!(ptr, length, 16, 15, 5, "☠️");
        assert_some_previous_valid_cutoff!(ptr, length, 16, 16, 5, "☠️");
        Ok(())
    }

    #[test]
    fn test_previous_valid_cutoff_at_various_indexes_4_bytes() -> Result<()> {
        let (ptr, length) = pointer::from_slice("smiley😀smiley".as_bytes())?;
        assert_none_previous_valid_cutoff!(ptr, length, 16, 0);
        assert_none_previous_valid_cutoff!(ptr, length, 16, 1);
        assert_none_previous_valid_cutoff!(ptr, length, 16, 2);
        assert_none_previous_valid_cutoff!(ptr, length, 16, 3);
        assert_none_previous_valid_cutoff!(ptr, length, 16, 4);
        assert_none_previous_valid_cutoff!(ptr, length, 16, 5);
        assert_some_previous_valid_cutoff!(ptr, length, 16, 6, 6, "😀");
        assert_some_previous_valid_cutoff!(ptr, length, 16, 7, 6, "😀");
        assert_some_previous_valid_cutoff!(ptr, length, 16, 8, 6, "😀");
        assert_some_previous_valid_cutoff!(ptr, length, 16, 9, 6, "😀");
        assert_some_previous_valid_cutoff!(ptr, length, 16, 10, 6, "😀");
        assert_some_previous_valid_cutoff!(ptr, length, 16, 11, 6, "😀");
        assert_some_previous_valid_cutoff!(ptr, length, 16, 12, 6, "😀");
        assert_some_previous_valid_cutoff!(ptr, length, 16, 13, 6, "😀");
        assert_some_previous_valid_cutoff!(ptr, length, 16, 14, 6, "😀");
        assert_some_previous_valid_cutoff!(ptr, length, 16, 15, 6, "😀");
        assert_some_previous_valid_cutoff!(ptr, length, 16, 16, 6, "😀");
        Ok(())
    }

    #[test]
    fn test_previous_valid_cutoff_at_various_indexes_ascii() -> Result<()> {
        let (ptr, length) = pointer::from_slice("abcdefghijklmnopqrstu".as_bytes())?;
        assert_none_previous_valid_cutoff!(ptr, length, 21, 7);

        Ok(())
    }

    #[test]
    fn test_previous_valid_cutoff_at_various_indexes_non_ascii() -> Result<()> {
        let (ptr, length) = pointer::from_slice("falcão🦅".as_bytes())?;
        assert_some_previous_valid_cutoff!(ptr, length, 11, 4, 4, "ã");
        assert_some_previous_valid_cutoff!(ptr, length, 11, 5, 4, "ã");
        assert_some_previous_valid_cutoff!(ptr, length, 11, 6, 4, "ã");
        assert_some_previous_valid_cutoff!(ptr, length, 11, 7, 7, "🦅");
        assert_some_previous_valid_cutoff!(ptr, length, 11, 8, 7, "🦅");
        assert_some_previous_valid_cutoff!(ptr, length, 11, 9, 7, "🦅");
        assert_some_previous_valid_cutoff!(ptr, length, 11, 10, 7, "🦅");
        assert_some_previous_valid_cutoff!(ptr, length, 11, 11, 7, "🦅");
        assert_some_previous_valid_cutoff!(ptr, length, 11, 12, 7, "🦅");
        assert_some_previous_valid_cutoff!(ptr, length, 11, 13, 7, "🦅");
        assert_some_previous_valid_cutoff!(ptr, length, 11, 14, 7, "🦅");
        assert_some_previous_valid_cutoff!(ptr, length, 11, 15, 7, "🦅");
        Ok(())
    }

    #[test]
    fn test_previous_valid_cutoff_at_first_index() -> Result<()> {
        let (ptr, length) = pointer::from_display("❤️🦅")?;
        assert_some_previous_valid_cutoff!(ptr, length, 10, 0, 0, "❤️");
        assert_some_previous_valid_cutoff!(ptr, length, 10, 1, 0, "❤️");
        assert_some_previous_valid_cutoff!(ptr, length, 10, 2, 0, "❤️");
        assert_some_previous_valid_cutoff!(ptr, length, 10, 3, 0, "❤️");
        assert_some_previous_valid_cutoff!(ptr, length, 10, 4, 0, "❤️");
        assert_some_previous_valid_cutoff!(ptr, length, 10, 5, 0, "❤️");
        assert_some_previous_valid_cutoff!(ptr, length, 10, 6, 6, "🦅");
        assert_some_previous_valid_cutoff!(ptr, length, 10, 7, 6, "🦅");
        assert_some_previous_valid_cutoff!(ptr, length, 10, 8, 6, "🦅");
        assert_some_previous_valid_cutoff!(ptr, length, 10, 9, 6, "🦅");
        assert_some_previous_valid_cutoff!(ptr, length, 10, 10, 6, "🦅");
        assert_some_previous_valid_cutoff!(ptr, length, 10, 11, 6, "🦅");
        assert_some_previous_valid_cutoff!(ptr, length, 10, 12, 6, "🦅");
        assert_some_previous_valid_cutoff!(ptr, length, 10, 13, 6, "🦅");
        assert_some_previous_valid_cutoff!(ptr, length, 10, 14, 6, "🦅");
        assert_some_previous_valid_cutoff!(ptr, length, 10, 15, 6, "🦅");
        Ok(())
    }

    #[test]
    fn test_previous_valid_cutoff_at_various_indexes_94_bytes() -> Result<()> {
        let (ptr, length) = pointer::from_display("👩🏻‍🚒👌🏿🧑🏽‍🚒👨‍🚒🌶️🎹💔🔥❤️‍🔥❤️‍🩹")?;
        assert_some_previous_valid_cutoff!(ptr, length, 94, 0, 0, "👩🏻‍🚒");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 1, 0, "👩🏻‍🚒");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 2, 0, "👩🏻‍🚒");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 3, 0, "👩🏻‍🚒");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 4, 4, "🏻\u{200d}🚒");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 5, 4, "🏻\u{200d}🚒");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 6, 4, "🏻\u{200d}🚒");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 7, 4, "🏻\u{200d}🚒");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 8, 4, "🏻\u{200d}🚒");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 9, 4, "🏻\u{200d}🚒");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 10, 4, "🏻\u{200d}🚒");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 11, 11, "🚒");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 12, 11, "🚒");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 13, 11, "🚒");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 14, 11, "🚒");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 15, 15, "👌🏿");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 16, 15, "👌🏿");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 17, 15, "👌🏿");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 18, 15, "👌🏿");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 19, 19, "🏿");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 20, 19, "🏿");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 21, 19, "🏿");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 22, 19, "🏿");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 23, 23, "🧑🏽‍🚒");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 24, 23, "🧑🏽‍🚒");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 25, 23, "🧑🏽‍🚒");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 26, 23, "🧑🏽‍🚒");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 27, 27, "🏽\u{200d}🚒");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 28, 27, "🏽\u{200d}🚒");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 29, 27, "🏽\u{200d}🚒");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 30, 27, "🏽\u{200d}🚒");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 31, 27, "🏽\u{200d}🚒");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 32, 27, "🏽\u{200d}🚒");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 33, 27, "🏽\u{200d}🚒");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 34, 34, "🚒👨\u{200d}🚒");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 35, 34, "🚒👨\u{200d}🚒");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 36, 34, "🚒👨\u{200d}🚒");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 36, 34, "🚒👨\u{200d}🚒");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 37, 34, "🚒👨\u{200d}🚒");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 38, 38, "👨\u{200d}🚒");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 39, 38, "👨\u{200d}🚒");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 40, 38, "👨\u{200d}🚒");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 41, 38, "👨\u{200d}🚒");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 42, 38, "👨\u{200d}🚒");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 43, 38, "👨\u{200d}🚒");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 44, 38, "👨\u{200d}🚒");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 45, 45, "🚒🌶\u{fe0f}");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 46, 45, "🚒🌶️");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 47, 45, "🚒🌶️");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 46, 45, "🚒🌶️");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 47, 45, "🚒🌶️");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 48, 45, "🚒🌶️");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 49, 49, "🌶️");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 50, 49, "🌶️");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 51, 49, "🌶️");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 52, 49, "🌶️");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 53, 49, "🌶️");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 54, 49, "🌶️");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 55, 49, "🌶️");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 56, 56, "🎹");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 57, 56, "🎹");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 58, 56, "🎹");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 59, 56, "🎹");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 60, 60, "💔");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 61, 60, "💔");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 62, 60, "💔");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 63, 60, "💔");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 64, 64, "🔥");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 65, 64, "🔥");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 66, 64, "🔥");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 67, 64, "🔥");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 68, 68, "❤️‍🔥");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 69, 68, "❤️‍🔥");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 70, 68, "❤️‍🔥");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 71, 68, "❤️‍🔥");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 72, 68, "❤️‍🔥");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 73, 68, "❤️‍🔥");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 74, 68, "❤️‍🔥");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 75, 68, "❤️‍🔥");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 76, 68, "❤️‍🔥");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 77, 77, "🔥");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 78, 77, "🔥");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 79, 77, "🔥");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 80, 77, "🔥");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 81, 81, "❤️‍🩹");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 82, 81, "❤️‍🩹");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 83, 81, "❤️‍🩹");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 84, 81, "❤️‍🩹");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 85, 81, "❤️‍🩹");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 86, 81, "❤️‍🩹");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 87, 81, "❤️‍🩹");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 88, 81, "❤️‍🩹");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 89, 81, "❤️‍🩹");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 90, 90, "🩹");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 91, 90, "🩹");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 92, 90, "🩹");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 93, 90, "🩹");
        assert_some_previous_valid_cutoff!(ptr, length, 94, 94, 90, "🩹");
        Ok(())
    }

    #[macro_export]
    macro_rules! assert_some_previous_valid_cutoff {
    (
        $ptr:expr,
        $length:expr,
        $expected_length:literal,
        $invalid_index:literal,
        $expected_valid_index:literal,
        $expected_rune_str:literal
        $(,)?
    ) => {{


        // debug_et_diagnostics::step!(fg=(line!() as u8), format!("expecting previous_valid_cutoff from invalid index {} to be {} matching rune \"{}\"", $invalid_index, $expected_valid_index, $expected_rune_str));

        assert_eq!($length, $expected_length, "expected length to be {} rather than {}", $expected_length, $length);
        let result = previous_valid_cutoff($ptr, $length, $invalid_index);
        assert!(result.is_some(), "expected previous_valid_cutoff at {} to not be None", $invalid_index);
        let actual = result.unwrap();
        assert_eq!(actual, $expected_valid_index, "expected previous_valid_cutoff to be {} rather than {}", $expected_valid_index, actual);
        let parts = RuneParts::from_raw_parts($ptr, $length);
        let result = parts.rune_at_index(actual);
        assert!(result.is_ok(), "expected valid Rune at index to be {} but got error: {}", $expected_valid_index, result.err().map(|err|err.to_string()).unwrap_or_default());
        let rune = result.unwrap();
        assert_eq!(rune.as_str(), $expected_rune_str,
                   "expected rune at index {} to match \"{}\" rather than \"{}\"",
                   actual, $expected_rune_str, rune.as_str());
    }};
}

    #[macro_export]
    macro_rules! assert_none_previous_valid_cutoff {
    (
        $ptr:expr, $length:expr, $expected_length:literal, $invalid_index:literal $(,)?
    ) => {{


        // debug_et_diagnostics::step!(fg = (line!() as u8),format!("expecting previous_valid_cutoff from invalid index {} to be None",$invalid_index));

        assert_eq!(
            $length, $expected_length,
            "expected length to be {} rather than {}",
            $expected_length, $length
        );
        let result = previous_valid_cutoff($ptr, $length, $invalid_index);
        assert!(
            result.is_none(),
            "expected previous_valid_cutoff at {} to not be None but is actually {:#?}",
            $invalid_index,
            result
        );
    }};
}
}
