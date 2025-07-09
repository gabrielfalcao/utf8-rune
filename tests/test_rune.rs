use utf8_rune::Rune;

#[test]
fn test_single_rune() {
    let rune = Rune::new("❤️");
    assert_eq!(rune.len(), 6);
    assert_eq!(rune.as_str(), "❤️");
    assert_eq!(rune.as_bytes(), "❤️".as_bytes());

    let rune = Rune::new("👌");
    assert_eq!(rune.len(), 4);
    assert_eq!(rune.as_str(), "👌");
    assert_eq!(rune.as_bytes(), "👌".as_bytes());

    let rune = Rune::new("👌🏻");
    assert_eq!(rune.len(), 8);
    assert_eq!(rune.as_str(), "👌🏻");
    assert_eq!(rune.as_bytes(), "👌🏻".as_bytes());

    let rune = Rune::new("👌🏼");
    assert_eq!(rune.len(), 8);
    assert_eq!(rune.as_str(), "👌🏼");
    assert_eq!(rune.as_bytes(), "👌🏼".as_bytes());

    let rune = Rune::new("👌🏽");
    assert_eq!(rune.len(), 8);
    assert_eq!(rune.as_str(), "👌🏽");
    assert_eq!(rune.as_bytes(), "👌🏽".as_bytes());

    let rune = Rune::new("👌🏾");
    assert_eq!(rune.len(), 8);
    assert_eq!(rune.as_str(), "👌🏾");
    assert_eq!(rune.as_bytes(), "👌🏾".as_bytes());

    let rune = Rune::new("👌🏿");
    assert_eq!(rune.len(), 8);
    assert_eq!(rune.len(), 8);
    assert_eq!(rune.as_str(), "👌🏿");
    assert_eq!(rune.as_bytes(), "👌🏿".as_bytes());
}

#[test]
fn test_from_multiple_runes() {
    let rune = Rune::new("👌👌🏻👌🏼👌🏽👌🏾👌🏿");
    assert_eq!(rune.len(), 4);
    assert_eq!(rune.as_str(), "👌");
    assert_eq!(rune.as_bytes(), "👌".as_bytes());

    let rune = Rune::new("👌🏻👌🏼👌🏽👌🏾👌🏿");
    assert_eq!(rune.len(), 8);
    assert_eq!(rune.as_str(), "👌🏻");
    assert_eq!(rune.as_bytes(), "👌🏻".as_bytes());

    let rune = Rune::new("👌🏼👌🏽👌🏾👌🏿");
    assert_eq!(rune.len(), 8);
    assert_eq!(rune.as_str(), "👌🏼");
    assert_eq!(rune.as_bytes(), "👌🏼".as_bytes());

    let rune = Rune::new("👌🏽👌🏾👌🏿");
    assert_eq!(rune.len(), 8);
    assert_eq!(rune.as_str(), "👌🏽");
    assert_eq!(rune.as_bytes(), "👌🏽".as_bytes());

    let rune = Rune::new("👌🏾👌🏿");
    assert_eq!(rune.len(), 8);
    assert_eq!(rune.as_str(), "👌🏾");
    assert_eq!(rune.as_bytes(), "👌🏾".as_bytes());

    let rune = Rune::new("👌🏿");
    assert_eq!(rune.len(), 8);
    assert_eq!(rune.as_str(), "👌🏿");
    assert_eq!(rune.as_bytes(), "👌🏿".as_bytes());
}
