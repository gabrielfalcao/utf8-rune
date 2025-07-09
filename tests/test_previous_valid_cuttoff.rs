#![allow(unused)]
use debug_et_diagnostics::{dbg_bytes, step};
use utf8_rune::{
    get_byte_slice_of, previous_valid_cutoff, slice_ptr_and_length_from_bytes, Result,
    RuneParts,
};

#[test]
fn test_previous_valid_cutoff_parts() -> Result<()> {
    let (ptr, length) = slice_ptr_and_length_from_bytes("👌👌🏻👌🏼👌🏽👌🏾👌🏿".as_bytes());
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
    let (ptr, length) = slice_ptr_and_length_from_bytes("❤️".as_bytes());
    assert_some_previous_valid_cutoff!(ptr, length, 6, 0, 0, "❤️");
    Ok(())
}

#[test]
fn test_previous_valid_cutoff_empty() -> Result<()> {
    let (ptr, length) = slice_ptr_and_length_from_bytes("".as_bytes());
    assert_none_previous_valid_cutoff!(ptr, length, 0, 0);

    Ok(())
}

#[test]
fn test_previous_valid_cutoff_at_various_indexes_6_bytes() -> Result<()> {
    // "☠️" => [0xe2, 0x98, 0xa0, 0xef, 0xb8, 0x8f] => [0b11100010, 0b10011000, 0b10100000, 0b11101111, 0b10111000, 0b10001111]
    let (ptr, length) = slice_ptr_and_length_from_bytes("skull☠️skull".as_bytes());
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
    // assert_none_previous_valid_cutoff!(ptr, length, 16, 16);
    assert_some_previous_valid_cutoff!(ptr, length, 16, 16, 5, "☠️");
    Ok(())
}

#[test]
fn test_previous_valid_cutoff_at_various_indexes_4_bytes() -> Result<()> {
    //  "😀" => [0xf0, 0x9f, 0x98, 0x80] => [0b11110000, 0b10011111, 0b10011000, 0b10000000]

    let (ptr, length) = slice_ptr_and_length_from_bytes("smiley😀smiley".as_bytes());
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
    // assert_none_previous_valid_cutoff!(ptr, length, 16, 16);
    Ok(())
}

#[test]
fn test_previous_valid_cutoff_at_various_indexes_ascii() -> Result<()> {
    let (ptr, length) =
        slice_ptr_and_length_from_bytes("abcdefghijklmnopqrstu".as_bytes());
    assert_none_previous_valid_cutoff!(ptr, length, 21, 7);

    Ok(())
}

#[test]
fn test_previous_valid_cutoff_at_various_indexes_non_ascii() -> Result<()> {
    // "🦅" => length=4 => [0xf0, 0x9f, 0xa6, 0x85] => [0b11110000, 0b10011111, 0b10100110, 0b10000101] => [240, 159, 166, 133]
    // "ã" => length=2 => [0xc3, 0xa3] => [0b11000011, 0b10100011] => [195, 163]

    let (ptr, length) = slice_ptr_and_length_from_bytes("falcão🦅".as_bytes());
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
    let (ptr, length) = slice_ptr_and_length_from_bytes("❤️🦅".as_bytes());
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
    let (ptr, length) =
        slice_ptr_and_length_from_bytes("👩🏻‍🚒👌🏿🧑🏽‍🚒👨‍🚒🌶️🎹💔🔥❤️‍🔥❤️‍🩹".as_bytes());
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
        use debug_et_diagnostics::{ansi, step};

        let line = line!() as u8;
        // step!(fg=line, format!("expecting previous_valid_cutoff from invalid index {} to be {} matching rune \"{}\"", $invalid_index, $expected_valid_index, $expected_rune_str));

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
        use debug_et_diagnostics::{ansi, step};

        let line = line!() as u8;
        // step!(
        //     fg = line,
        //     format!(
        //         "expecting previous_valid_cutoff from invalid index {} to be None",
        //         $invalid_index
        //     )
        // );

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
