/// Returns names from `candidates` that fuzzy-match `needle`.
///
/// Two-pass strategy:
///   1. Case-insensitive substring — "wo" matches "work"
///   2. Case-insensitive subsequence — "wrk" matches "work" (if pass 1 is empty)
pub fn find_matches<'a, S: AsRef<str>>(needle: &str, candidates: &'a [S]) -> Vec<&'a str> {
    let needle_lc = needle.to_lowercase();

    // Pre-compute lowercased candidates once for both passes.
    let lowered: Vec<String> = candidates
        .iter()
        .map(|s| s.as_ref().to_lowercase())
        .collect();

    let subs: Vec<&str> = candidates
        .iter()
        .zip(lowered.iter())
        .filter(|(_, lc)| lc.contains(&needle_lc))
        .map(|(s, _)| s.as_ref())
        .collect();

    if !subs.is_empty() {
        return subs;
    }

    candidates
        .iter()
        .zip(lowered.iter())
        .filter(|(_, lc)| is_subsequence(&needle_lc, lc))
        .map(|(s, _)| s.as_ref())
        .collect()
}

fn is_subsequence(needle: &str, haystack: &str) -> bool {
    let mut it = haystack.chars();
    needle.chars().all(|c| it.any(|h| h == c))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn exact_match() {
        let c = names(&["work", "api", "home"]);
        assert_eq!(find_matches("work", &c), vec!["work"]);
    }

    #[test]
    fn substring_match() {
        let c = names(&["mywork", "api", "homework"]);
        let m = find_matches("work", &c);
        assert_eq!(m, vec!["mywork", "homework"]);
    }

    #[test]
    fn case_insensitive_substring() {
        let c = names(&["Work", "API"]);
        assert_eq!(find_matches("work", &c), vec!["Work"]);
    }

    #[test]
    fn subsequence_fallback() {
        let c = names(&["work", "api"]);
        assert_eq!(find_matches("wrk", &c), vec!["work"]);
    }

    #[test]
    fn no_match_returns_empty() {
        let c = names(&["work", "api"]);
        assert!(find_matches("zzz", &c).is_empty());
    }

    #[test]
    fn substring_takes_precedence_over_subsequence() {
        // "ap" is a substring of "api"; "ap" is also a subsequence of "muxx-app"
        // but since substring pass finds matches it should not fall through
        let c = names(&["api", "muxx-app"]);
        let m = find_matches("ap", &c);
        assert!(m.contains(&"api"));
        assert!(m.contains(&"muxx-app")); // "ap" is also a substring of "muxx-app"
    }

    #[test]
    fn empty_needle_matches_all_via_substring() {
        // Empty string is a substring of every string
        let c = names(&["alpha", "beta", "gamma"]);
        let m = find_matches("", &c);
        assert_eq!(m.len(), 3);
    }

    #[test]
    fn empty_candidates_returns_empty() {
        let c: Vec<String> = vec![];
        assert!(find_matches("foo", &c).is_empty());
    }

    #[test]
    fn all_candidates_match_substring() {
        let c = names(&["foobar", "foo", "prefoo"]);
        let m = find_matches("foo", &c);
        assert_eq!(m.len(), 3);
    }

    #[test]
    fn subsequence_skips_non_matching_candidate() {
        // "aa" is a subsequence of "alpha" and "gamma" but not "beta" (only one 'a')
        let c = names(&["alpha", "beta", "gamma"]);
        let m = find_matches("aa", &c);
        assert!(m.contains(&"alpha"));
        assert!(m.contains(&"gamma"));
        assert!(!m.contains(&"beta"));
    }

    #[test]
    fn case_insensitive_subsequence_fallback() {
        // "mp" is a subsequence of "MyProject" (case-insensitive)
        let c = names(&["MyProject"]);
        let m = find_matches("mp", &c);
        assert_eq!(m, vec!["MyProject"]);
    }

    #[test]
    fn single_char_needle_substring() {
        let c = names(&["api", "work", "zoo"]);
        let m = find_matches("a", &c);
        assert_eq!(m, vec!["api"]);
    }

    #[test]
    fn needle_longer_than_candidate_no_match() {
        let c = names(&["hi"]);
        assert!(find_matches("hello", &c).is_empty());
    }

    #[test]
    fn subsequence_full_word() {
        // "abc" as subsequence of "aXbXc"
        let c = names(&["axbxc"]);
        let m = find_matches("abc", &c);
        assert_eq!(m, vec!["axbxc"]);
    }

    #[test]
    fn subsequence_not_matching_order() {
        // "ba" is NOT a subsequence of "ab" (b comes after a)
        // wait, "ab": iterator starts at 'a', needle 'b' → skip 'a', then 'b' found
        // Actually "ba": needle 'b' → skip 'a' → no match. So empty.
        let c = names(&["ab"]);
        // "ba" as subsequence of "ab": look for 'b' first → 'a' no, 'b' yes → found
        // then look for 'a' → no more chars → false
        assert!(find_matches("ba", &c).is_empty());
    }

    #[test]
    fn hyphenated_session_names_substring_match() {
        let c = names(&["my-cool-project", "my-other-thing", "unrelated"]);
        let m = find_matches("cool", &c);
        assert_eq!(m, vec!["my-cool-project"]);
    }
}
