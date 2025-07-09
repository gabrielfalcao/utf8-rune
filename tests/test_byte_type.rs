use utf8_rune::ByteType;

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
