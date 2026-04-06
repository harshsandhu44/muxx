/// Convert a raw string or path into a safe tmux session name.
///
/// Rules applied in order:
///   1. Extract the last non-empty path segment if the input contains '/'
///   2. Lowercase
///   3. Trim whitespace
///   4. Replace spaces with hyphens
///   5. Replace invalid characters (not a-z, 0-9, hyphen, underscore) with hyphens
///   6. Collapse repeated hyphens
///   7. Remove leading/trailing hyphens
pub fn sanitize_session_name(input: &str) -> String {
    // Step 1: extract basename if path-like (contains '/')
    let base = if input.contains('/') {
        input.split('/').rfind(|s| !s.is_empty()).unwrap_or(input)
    } else {
        input
    };

    // Steps 2-7: lowercase, trim, sanitize chars, collapse hyphens, strip edges
    let lowered = base.trim().to_lowercase();

    let mut result = String::with_capacity(lowered.len());
    let mut last_was_hyphen = false;

    for ch in lowered.chars() {
        if ch == ' ' || ch == '-' || (!ch.is_ascii_alphanumeric() && ch != '_') {
            if !last_was_hyphen && !result.is_empty() {
                result.push('-');
                last_was_hyphen = true;
            }
        } else {
            result.push(ch);
            last_was_hyphen = false;
        }
    }

    // Strip trailing hyphen
    result.trim_end_matches('-').to_string()
}

/// Resolve the effective session name, letting an explicit override win.
/// Falls back to sanitizing the raw input when no override is provided.
pub fn resolve_session_name(raw: &str, override_name: Option<&str>) -> String {
    match override_name {
        Some(o) if !o.trim().is_empty() => sanitize_session_name(o),
        _ => sanitize_session_name(raw),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_basename_from_absolute_path() {
        assert_eq!(
            sanitize_session_name("/Users/harsh/Code/my-project"),
            "my-project"
        );
    }

    #[test]
    fn extracts_basename_from_path_with_trailing_slash() {
        assert_eq!(sanitize_session_name("/tmp/"), "tmp");
    }

    #[test]
    fn lowercases_input() {
        assert_eq!(sanitize_session_name("MyProject"), "myproject");
    }

    #[test]
    fn preserves_underscores() {
        assert_eq!(sanitize_session_name("UPPER_CASE"), "upper_case");
    }

    #[test]
    fn replaces_spaces_with_hyphens() {
        assert_eq!(sanitize_session_name("My Cool App"), "my-cool-app");
    }

    #[test]
    fn trims_leading_and_trailing_whitespace() {
        assert_eq!(sanitize_session_name("  hello world  "), "hello-world");
    }

    #[test]
    fn replaces_invalid_characters_with_hyphens() {
        assert_eq!(sanitize_session_name("foo@bar!baz"), "foo-bar-baz");
    }

    #[test]
    fn collapses_repeated_hyphens() {
        assert_eq!(sanitize_session_name("foo---bar"), "foo-bar");
    }

    #[test]
    fn removes_leading_and_trailing_hyphens() {
        assert_eq!(sanitize_session_name("-foo-"), "foo");
    }

    #[test]
    fn single_segment_not_treated_as_path() {
        assert_eq!(sanitize_session_name("myapp"), "myapp");
    }

    #[test]
    fn two_segment_path_extracts_last_segment() {
        assert_eq!(sanitize_session_name("Code/my-project"), "my-project");
    }

    #[test]
    fn resolve_sanitizes_raw_when_no_override() {
        assert_eq!(resolve_session_name("/home/user/my app", None), "my-app");
    }

    #[test]
    fn resolve_uses_override_when_provided() {
        assert_eq!(
            resolve_session_name("/home/user/my app", Some("Custom Name")),
            "custom-name"
        );
    }

    #[test]
    fn resolve_ignores_blank_override() {
        assert_eq!(
            resolve_session_name("/home/user/myapp", Some("   ")),
            "myapp"
        );
    }

    #[test]
    fn resolve_override_with_no_raw_path() {
        assert_eq!(
            resolve_session_name("ignored", Some("My Override")),
            "my-override"
        );
    }

    #[test]
    fn numbers_only_preserved() {
        assert_eq!(sanitize_session_name("12345"), "12345");
    }

    #[test]
    fn mixed_alphanumeric_preserved() {
        assert_eq!(sanitize_session_name("project2024"), "project2024");
    }

    #[test]
    fn consecutive_special_chars_collapse_to_single_hyphen() {
        assert_eq!(sanitize_session_name("foo@@@bar"), "foo-bar");
    }

    #[test]
    fn path_with_multiple_consecutive_trailing_slashes() {
        assert_eq!(sanitize_session_name("/tmp/foo///"), "foo");
    }

    #[test]
    fn deep_absolute_path_extracts_basename() {
        assert_eq!(sanitize_session_name("/a/b/c/d/my-project"), "my-project");
    }

    #[test]
    fn leading_special_chars_dropped() {
        // '@' before any real char: hyphen suppressed because result is empty
        assert_eq!(sanitize_session_name("@@@foo"), "foo");
    }

    #[test]
    fn only_special_chars_produces_empty_string() {
        // All chars become hyphens, but leading hyphen suppression + trailing strip → ""
        assert_eq!(sanitize_session_name("@@@"), "");
    }

    #[test]
    fn unicode_non_ascii_replaced_with_hyphen() {
        // 'é' is not ASCII alphanumeric → treated as invalid char → hyphen → stripped trailing
        assert_eq!(sanitize_session_name("café"), "caf");
    }

    #[test]
    fn underscore_preserved_alongside_hyphens() {
        assert_eq!(sanitize_session_name("foo_bar-baz"), "foo_bar-baz");
    }

    #[test]
    fn mixed_case_with_numbers_and_special() {
        // '.' and '!' become hyphens; "v2.0!" → "v2-0"
        assert_eq!(sanitize_session_name("My App v2.0!"), "my-app-v2-0");
    }

    #[test]
    fn single_char_alphanumeric() {
        assert_eq!(sanitize_session_name("a"), "a");
    }

    #[test]
    fn single_char_special() {
        assert_eq!(sanitize_session_name("@"), "");
    }

    #[test]
    fn path_single_slash() {
        // "/" splits into ["", ""] — rfind non-empty finds nothing, falls back to input "/"
        // then "/" is treated as special char → ""
        assert_eq!(sanitize_session_name("/"), "");
    }

    #[test]
    fn resolve_sanitizes_override_with_special_chars() {
        assert_eq!(
            resolve_session_name("anything", Some("My App v2!")),
            "my-app-v2"
        );
    }

    #[test]
    fn resolve_empty_raw_no_override() {
        assert_eq!(resolve_session_name("", None), "");
    }
}
