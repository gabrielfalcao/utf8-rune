#![allow(unused)]
use utf8_rune::{
    get_rune_cutoff_at_index, slice_ptr_and_length_from_bytes, Result, RuneParts,
};

#[test]
fn test_get_rune_cutoff_at_first_index_single_rune() -> Result<()> {
    let (ptr, length) = slice_ptr_and_length_from_bytes("❤️".as_bytes());
    assert_get_rune_cutoff_at_index!(ptr, length, 6, 0, 6, "❤️");
    Ok(())
}

#[test]
fn test_get_rune_cutoff_empty() -> Result<()> {
    let (ptr, length) = slice_ptr_and_length_from_bytes("".as_bytes());
    assert_get_rune_cutoff_at_index!(ptr, length, 0, 0, 0, "");
    Ok(())
}

#[test]
fn test_get_rune_cutoff_at_various_indexes_4_bytes() -> Result<()> {
    //  "😀" => [0xf0, 0x9f, 0x98, 0x80] => [0b11110000, 0b10011111, 0b10011000, 0b10000000]
    let (ptr, length) = slice_ptr_and_length_from_bytes("smiley😀smiley".as_bytes());
    assert_get_rune_cutoff_at_index!(ptr, length, 16, 6, 10, "😀");

    Ok(())
}

#[test]
fn test_get_rune_cutoff_at_various_indexes_6_bytes() -> Result<()> {
    // "☠️" => [0xe2, 0x98, 0xa0, 0xef, 0xb8, 0x8f] => [0b11100010, 0b10011000, 0b10100000, 0b11101111, 0b10111000, 0b10001111]
    let (ptr, length) = slice_ptr_and_length_from_bytes("skull☠️skull".as_bytes());
    assert_get_rune_cutoff_at_index!(ptr, length, 16, 5, 11, "☠️");

    Ok(())
}

#[test]
fn test_get_rune_cutoff_at_various_indexes_ascii() -> Result<()> {
    let (ptr, length) =
        slice_ptr_and_length_from_bytes("abcdefghijklmnopqrstu".as_bytes());
    assert_get_rune_cutoff_at_index!(ptr, length, 21, 7, 8, "h");

    Ok(())
}

#[test]
fn test_get_rune_cutoff_at_various_indexes_non_ascii() -> Result<()> {
    // "🦅" => length=4 => [0xf0, 0x9f, 0xa6, 0x85] => [0b11110000, 0b10011111, 0b10100110, 0b10000101] => [240, 159, 166, 133]
    // "ã" => length=2 => [0xc3, 0xa3] => [0b11000011, 0b10100011] => [195, 163]

    let (ptr, length) = slice_ptr_and_length_from_bytes("falcão🦅".as_bytes());
    assert_get_rune_cutoff_at_index!(ptr, length, 11, 4, 6, "ã");
    assert_get_rune_cutoff_at_index!(ptr, length, 11, 6, 7, "o");
    assert_get_rune_cutoff_at_index!(ptr, length, 11, 7, 11, "🦅");
    Ok(())
}

#[test]
fn test_get_rune_cutoff_at_first_index() -> Result<()> {
    let (ptr, length) = slice_ptr_and_length_from_bytes("❤️🦅".as_bytes());
    assert_get_rune_cutoff_at_index!(ptr, length, 10, 0, 6, "❤️");
    assert_get_rune_cutoff_at_index!(ptr, length, 10, 6, 10, "🦅");
    Ok(())
}

#[test]
fn test_get_rune_cutoff_unexpected_continuation_byte() -> Result<()> {
    let (ptr, length) = slice_ptr_and_length_from_bytes("❤️🦅".as_bytes());
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
    let (ptr, length) =
        slice_ptr_and_length_from_bytes("👩🏻‍🚒👌🏿🧑🏽‍🚒👨‍🚒🌶️🎹💔🔥❤️‍🔥❤️‍🩹".as_bytes());
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
        use debug_et_diagnostics::{ansi, fore, from_bytes, indent, step};
        use utf8_rune::{get_byte_slice_of, format_bytes};

        let line = line!() as u8;
        // step!(fg=line, format!("expecting {} from index..cutoff {}..{}", $expected, $index, $cutoff));

        let slice = utf8_rune::get_byte_slice_of($ptr, 0, $length)
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
                Ok(actual) => actual.as_str()
            .to_string(),
                Err(error) => {
                    panic!("{}:{} RuneParts::from_raw_parts({:#?}, {})", file!(), line!(), $ptr, $length);
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
    use debug_et_diagnostics::color::{ansi, byte_hex, fore};
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
