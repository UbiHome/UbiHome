use regex::Regex;
use std::sync::LazyLock;

#[macro_export]
macro_rules! regex_pair {
    ($name:ident, $remover_name:ident, $pattern:literal) => {
        pub static $name: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(concat!("^", $pattern, "$")).unwrap());
        pub static $remover_name: LazyLock<Regex> = LazyLock::new(|| Regex::new($pattern).unwrap());
    };
}

regex_pair!(ID_RE, ID_RE_REMOVER, r"[a-zA-Z0-9-_]*");

#[test]
fn test_id_re_matches() {
    // Matches
    assert!(ID_RE.is_match("test_a_1"));
    assert!(ID_RE.is_match("test"));
    assert!(ID_RE.is_match("test-123"));

    // No Matches
    assert!(!ID_RE.is_match("test&"));
    assert!(!ID_RE.is_match("test😀"));
}

pub fn is_id_string_option(value: &Option<String>, _: &()) -> garde::Result {
    if let Some(inner_value) = value
        && !ID_RE.is_match(inner_value)
    {
        let invalid_values = ID_RE_REMOVER.replace_all(inner_value, "");
        return Err(garde::Error::new(format!(
            "ID must only contain letters, numbers, hyphens, and underscores, but found: {}",
            invalid_values
        )));
    }
    Ok(())
}

regex_pair!(READABLE_RE, READABLE_RE_REMOVER, r"[ -~\p{L}\p{M}*]*");

#[test]
fn test_readable_re_matches() {
    // Matches
    assert!(READABLE_RE.is_match("test_a_1"));
    assert!(READABLE_RE.is_match("test"));
    assert!(READABLE_RE.is_match("test-123"));
    assert!(READABLE_RE.is_match("test&"));
    assert!(READABLE_RE.is_match("Hello, 世界"));

    // Debatable
    assert!(!READABLE_RE.is_match("test😀"));

    // No Matches
    assert!(!READABLE_RE.is_match("\0")); // null character
    assert!(!READABLE_RE.is_match("\u{200B}")); // zero-width space
    assert!(!READABLE_RE.is_match("\u{2800}")); // braille pattern blank
}

pub fn readable_string_error(value: &str) -> garde::Error {
    let non_printable_values = READABLE_RE_REMOVER.replace_all(value, "");
    garde::Error::new(format!(
        "This string contains non-printable characters: {:#?}. You can only use readable characters.",
        non_printable_values
    ))
}

pub fn is_readable_string(value: &str, _: &()) -> garde::Result {
    if !READABLE_RE.is_match(value) {
        return Err(readable_string_error(value));
    }
    Ok(())
}

pub fn is_readable_string_option(value: &Option<String>, _: &()) -> garde::Result {
    if let Some(inner_value) = value
        && !READABLE_RE.is_match(inner_value)
    {
        return Err(readable_string_error(inner_value));
    }
    Ok(())
}
