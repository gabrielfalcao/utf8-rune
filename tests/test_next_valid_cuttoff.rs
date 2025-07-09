#![allow(unused)]
use debug_et_diagnostics::{dbg_bytes, step};
use utf8_rune::{
    get_byte_slice_of, next_valid_cutoff, slice_ptr_and_length_from_bytes, Result,
    RuneParts,
};

#[test]
fn test_next_valid_cutoff_parts() -> Result<()> {
    let (ptr, length) = slice_ptr_and_length_from_bytes("👌👌🏻👌🏼👌🏽👌🏾👌🏿".as_bytes());
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
    let (ptr, length) = slice_ptr_and_length_from_bytes("❤️".as_bytes());
    assert_some_next_valid_cutoff!(ptr, length, 6, 0, 0, "❤️");
    Ok(())
}

#[test]
fn test_next_valid_cutoff_empty() -> Result<()> {
    let (ptr, length) = slice_ptr_and_length_from_bytes("".as_bytes());
    assert_none_next_valid_cutoff!(ptr, length, 0, 0);

    Ok(())
}

#[test]
fn test_next_valid_cutoff_at_various_indexes_6_bytes() -> Result<()> {
    // "☠️" => [0xe2, 0x98, 0xa0, 0xef, 0xb8, 0x8f] => [0b11100010, 0b10011000, 0b10100000, 0b11101111, 0b10111000, 0b10001111]
    let (ptr, length) = slice_ptr_and_length_from_bytes("skull☠️skull".as_bytes());
    assert_some_next_valid_cutoff!(ptr, length, 16, 0, 0, "s");
    assert_some_next_valid_cutoff!(ptr, length, 16, 4, 4, "l");
    assert_some_next_valid_cutoff!(ptr, length, 16, 5, 5, "☠️");
    Ok(())
}

#[test]
fn test_next_valid_cutoff_at_various_indexes_4_bytes() -> Result<()> {
    //  "😀" => [0xf0, 0x9f, 0x98, 0x80] => [0b11110000, 0b10011111, 0b10011000, 0b10000000]

    let (ptr, length) = slice_ptr_and_length_from_bytes("smiley😀smiley".as_bytes());
    assert_some_next_valid_cutoff!(ptr, length, 16, 5, 5, "y");
    assert_some_next_valid_cutoff!(ptr, length, 16, 6, 6, "😀");
    Ok(())
}

#[test]
fn test_next_valid_cutoff_at_various_indexes_ascii() -> Result<()> {
    let (ptr, length) =
        slice_ptr_and_length_from_bytes("abcdefghijklmnopqrstu".as_bytes());
    assert_some_next_valid_cutoff!(ptr, length, 21, 7, 7, "h");

    Ok(())
}

#[test]
fn test_next_valid_cutoff_at_various_indexes_non_ascii() -> Result<()> {
    // "🦅" => length=4 => [0xf0, 0x9f, 0xa6, 0x85] => [0b11110000, 0b10011111, 0b10100110, 0b10000101] => [240, 159, 166, 133]
    // "ã" => length=2 => [0xc3, 0xa3] => [0b11000011, 0b10100011] => [195, 163]

    let (ptr, length) = slice_ptr_and_length_from_bytes("falcão🦅".as_bytes());
    assert_some_next_valid_cutoff!(ptr, length, 11, 4, 4, "ã");
    assert_some_next_valid_cutoff!(ptr, length, 11, 5, 6, "o");
    assert_some_next_valid_cutoff!(ptr, length, 11, 6, 6, "o");
    assert_some_next_valid_cutoff!(ptr, length, 11, 7, 7, "🦅");
    assert_none_next_valid_cutoff!(ptr, length, 11, 8);
    Ok(())
}

#[test]
fn test_next_valid_cutoff_at_first_index() -> Result<()> {
    let (ptr, length) = slice_ptr_and_length_from_bytes("❤️🦅".as_bytes());
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
    let (ptr, length) =
        slice_ptr_and_length_from_bytes("👩🏻‍🚒👌🏿🧑🏽‍🚒👨‍🚒🌶️🎹💔🔥❤️‍🔥❤️‍🩹".as_bytes());
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
        use debug_et_diagnostics::{ansi, step};

        let line = line!() as u8;
        // step!(fg=line, format!("expecting next_valid_cutoff from invalid index {} to be {} matching rune \"{}\"", $invalid_index, $expected_valid_index, $expected_rune_str));

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
        $ptr:expr, $length:expr, $expected_length:literal, $invalid_index:literal $(,)?
    ) => {{
        use debug_et_diagnostics::{ansi, step};

        let line = line!() as u8;
        // step!(
        //     fg = line,
        //     format!(
        //         "expecting next_valid_cutoff from invalid index {} to be None",
        //         $invalid_index
        //     )
        // );

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
