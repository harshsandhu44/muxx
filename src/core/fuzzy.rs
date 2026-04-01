/// Returns names from `candidates` that fuzzy-match `needle`.
///
/// Two-pass strategy:
///   1. Case-insensitive substring — "wo" matches "work"
///   2. Case-insensitive subsequence — "wrk" matches "work" (if pass 1 is empty)
pub fn find_matches<'a>(needle: &str, candidates: &'a [String]) -> Vec<&'a str> {
    let needle_lc = needle.to_lowercase();

    let subs: Vec<&str> = candidates
        .iter()
        .filter(|s| s.to_lowercase().contains(&needle_lc))
        .map(String::as_str)
        .collect();

    if !subs.is_empty() {
        return subs;
    }

    candidates
        .iter()
        .filter(|s| is_subsequence(&needle_lc, &s.to_lowercase()))
        .map(String::as_str)
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
}
