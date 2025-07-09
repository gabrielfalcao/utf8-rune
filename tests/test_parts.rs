use utf8_rune::{Rune, RuneParts};

#[test]
fn test_rune_at_index_error() {
    let parts = RuneParts::new("👌👌🏻👌🏼👌🏽👌🏾👌🏿");
    {
        let result = parts.rune_at_index(1); // Ok("👌")
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert_eq!(err.previous_valid_cutoff(), Some(0));
        assert_eq!(err.next_valid_cutoff(), Some(4));
    }
    {
        let result = parts.rune_at_index(5); // Ok("👌🏻")
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert_eq!(err.previous_valid_cutoff(), Some(4));
        assert_eq!(err.next_valid_cutoff(), Some(8));
    }
    {
        let result = parts.rune_at_index(13); // Ok("👌🏼")
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert_eq!(err.previous_valid_cutoff(), Some(12));
        assert_eq!(err.next_valid_cutoff(), Some(16));
    }
    {
        let result = parts.rune_at_index(21); // Ok("👌🏽")
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert_eq!(err.previous_valid_cutoff(), Some(20));
        assert_eq!(err.next_valid_cutoff(), Some(24));
    }
    {
        let result = parts.rune_at_index(29); // Ok("👌🏾")
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert_eq!(err.previous_valid_cutoff(), Some(28));
        assert_eq!(err.next_valid_cutoff(), Some(32));
    }

    {
        let result = parts.rune_at_index(37); // Ok("👌🏿")
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert_eq!(err.previous_valid_cutoff(), Some(36));
        assert_eq!(err.next_valid_cutoff(), Some(40));
    }
}

#[test]
fn test_new_single_rune() {
    let parts = RuneParts::new("❤️");
    assert_eq!(parts.len(), 6);
    assert_eq!(parts.as_str(), "❤️");
    assert_eq!(parts.as_bytes(), "❤️".as_bytes());
}
#[test]
fn test_new_multiple_runes() {
    let parts = RuneParts::new("👌👌🏻👌🏼👌🏽👌🏾👌🏿");
    assert_eq!(parts.len(), 44);
    assert_eq!(parts.as_str(), "👌👌🏻👌🏼👌🏽👌🏾👌🏿");
    assert_eq!(parts.as_bytes(), "👌👌🏻👌🏼👌🏽👌🏾👌🏿".as_bytes());
}

#[test]
fn test_rune_indexes() {
    let parts = RuneParts::new("👌👌🏻👌🏼👌🏽👌🏾👌🏿");
    assert_eq!(parts.indexes(), vec![0, 4, 12, 20, 28, 36, 44]);
}
#[test]
fn test_rune_at_index() {
    let parts = RuneParts::new("👌👌🏻👌🏼👌🏽👌🏾👌🏿");
    assert_eq!(parts.rune_at_index(0), Ok(Rune::new("👌")));
    assert_eq!(parts.rune_at_index(4), Ok(Rune::new("👌🏻")));
    assert_eq!(parts.rune_at_index(12), Ok(Rune::new("👌🏼")));
    assert_eq!(parts.rune_at_index(20), Ok(Rune::new("👌🏽")));
    assert_eq!(parts.rune_at_index(28), Ok(Rune::new("👌🏾")));
    assert_eq!(parts.rune_at_index(36), Ok(Rune::new("👌🏿")));
}
