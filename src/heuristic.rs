use crate::{
    continuation_bytes_location, copy_ptr, get_byte_at_index, get_byte_slice_of,
    is_valid_utf8_str_of, ByteType, Error, Result,
};
#[inline]
pub fn split_at_first_rune<'g>(ptr: *const u8, length: usize) -> usize {
    get_rune_cutoff_at_index(ptr, length, 0).expect("should not fail at index 0")
}

#[inline]
pub fn get_rune_cutoff_at_index<'g>(
    ptr: *const u8,
    length: usize,
    index: usize,
) -> Result<usize> {
    let ptr = copy_ptr(ptr, length);

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
                if is_valid_utf8_str_of(ptr, index, tcutoff-index) {
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

pub fn unexpected_continuation_byte_at_index_error<'e>(
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

#[inline]
pub fn previous_valid_cutoff<'e>(
    ptr: *const u8,
    length: usize,
    index: usize,
) -> Option<usize> {
    let ptr = copy_ptr(ptr, length);
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
    let ptr = copy_ptr(ptr, length);
    let mut next_index = index;

    while next_index < length {
        if let Some((count, _ty_)) = continuation_bytes_location(ptr, length, next_index)
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
