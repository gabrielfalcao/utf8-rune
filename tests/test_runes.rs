use utf8_rune::{Result, Runes};

#[test]
fn test_runes() -> Result<()> {
    let parts = Runes::new("👩🏻‍🚒👌🏿🧑🏽‍🚒👨‍🚒🌶️🎹💔🔥❤️‍🔥❤️‍🩹");
    assert_eq!(
        parts
            .runes()?
            .iter()
            .map(|rune| rune.to_string())
            .collect::<Vec<String>>(),
        vec![
            "👩🏻‍🚒",
            "👌🏿",
            "🧑🏽‍🚒",
            "👨‍🚒",
            "🌶️",
            "🎹",
            "💔",
            "🔥",
            "❤️‍🔥",
            "❤️‍🩹",
        ]
    );
    Ok(())
}
