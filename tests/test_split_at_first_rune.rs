use utf8_rune::{slice_ptr_and_length_from_bytes, split_at_first_rune, Result};

#[test]
fn test_split_at_first_rune_4_bytes() -> Result<()> {
    //  "😀" => [0xf0, 0x9f, 0x98, 0x80] => [0b11110000, 0b10011111, 0b10011000, 0b10000000]
    let (ptr, length) = slice_ptr_and_length_from_bytes("😀smiley".as_bytes());
    let cutoff = split_at_first_rune(ptr, length);
    assert_eq!(cutoff, 4);
    assert_eq!(length, 10);

    Ok(())
}

#[test]
fn test_split_at_first_rune_6_bytes() -> Result<()> {
    // "☠️" => [0xe2, 0x98, 0xa0, 0xef, 0xb8, 0x8f] => [0b11100010, 0b10011000, 0b10100000, 0b11101111, 0b10111000, 0b10001111]
    let (ptr, length) = slice_ptr_and_length_from_bytes("☠️skull".as_bytes());
    let cutoff = split_at_first_rune(ptr, length);
    assert_eq!(length, 11);
    assert_eq!(cutoff, 6);

    Ok(())
}

#[test]
fn test_split_at_first_ascii() -> Result<()> {
    let (ptr, length) = slice_ptr_and_length_from_bytes("abcdefghijklmnopqrstu".as_bytes());
    let cutoff = split_at_first_rune(ptr, length);
    assert_eq!(cutoff, 1);
    assert_eq!(length, 21);

    Ok(())
}

#[test]
fn test_split_at_first_nonascii_single_byte_character() -> Result<()> {
    let (ptr, length) = slice_ptr_and_length_from_bytes("ão".as_bytes());
    let cutoff = split_at_first_rune(ptr, length);
    assert_eq!(cutoff, 2);
    assert_eq!(length, 3);

    Ok(())
}


#[test]
fn test_split_at_first_rune_single_heart() -> Result<()> {
    let (ptr, length) = slice_ptr_and_length_from_bytes("❤️".as_bytes());
    let cutoff = split_at_first_rune(ptr, length);
    assert_eq!(cutoff, 6);
    assert_eq!(length, 6);
    Ok(())
}

#[test]
fn test_split_at_first_rune_two_runes() -> Result<()> {
    let (ptr, length) = slice_ptr_and_length_from_bytes("❤️🦅".as_bytes());
    let cutoff = split_at_first_rune(ptr, length);
    assert_eq!(cutoff, 6);
    assert_eq!(length, 10);
    Ok(())
}
