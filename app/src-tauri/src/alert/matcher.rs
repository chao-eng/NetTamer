//! Process-name matcher supporting `*` wildcards (case-insensitive).

/// Returns `true` if `name` matches `pattern`.
///
/// Matching is case-insensitive. A pattern without `*` requires an exact match
/// (also allowing a `.exe` suffix on either side). A pattern containing `*`
/// performs ordered segment matching (e.g. `chrome*`, `*setup*`, `svc*host`).
pub fn match_process(pattern: &str, name: &str) -> bool {
    let pattern = pattern.to_lowercase();
    let name = name.to_lowercase();

    if !pattern.contains('*') {
        return pattern == name
            || pattern == format!("{}.exe", name)
            || format!("{}.exe", pattern) == name;
    }

    let parts: Vec<&str> = pattern.split('*').collect();
    let mut pos = 0usize;
    let len = parts.len();

    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if i == 0 {
            if !name.starts_with(part) {
                return false;
            }
            pos = part.len();
        } else if i == len - 1 {
            if !name[pos..].ends_with(part) {
                return false;
            }
        } else {
            match name[pos..].find(part) {
                Some(idx) => pos += idx + part.len(),
                None => return false,
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::match_process;

    #[test]
    fn exact_match() {
        assert!(match_process("chrome.exe", "chrome.exe"));
        assert!(match_process("chrome", "chrome.exe"));
    }

    #[test]
    fn wildcard_prefix() {
        assert!(match_process("chrome*", "chrome.exe"));
        assert!(match_process("chrom*", "chromium.exe"));
        assert!(!match_process("chrome*", "firefox"));
    }

    #[test]
    fn wildcard_substring() {
        assert!(match_process("*host*", "svchost.exe"));
        assert!(!match_process("*host*", "explorer.exe"));
    }
}
