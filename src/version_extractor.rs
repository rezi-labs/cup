use once_cell::sync::Lazy;
use regex::Regex;

/// Represents different formats used when replacing versions in text
#[derive(Clone, Copy)]
pub enum ReplacementFormat {
    Simple,   // format!("{}{}{}", &caps[1], new_version, &caps[3])
    Extended, // format!("{}{}{}{}", &caps[1], new_version, &caps[4], &caps[5])
    Quoted,   // format!("{}{}{}{}", &caps[1], new_version, &caps[3], &caps[4])
}

/// Configuration for a version pattern, containing regex patterns for extraction and replacement
pub struct VersionPattern {
    pub replace_regex: &'static Lazy<Regex>,
    pub replacement_format: ReplacementFormat,
}

// Lazy static regex patterns compiled only once at startup
// Pattern 1: name = version // comment or name = version # comment
static VERSION_REPLACE_RE_1: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\w+\s*=\s*)([0-9]*\.?[0-9]+(?:\.[0-9]+)*)(\s*(?://|#).*)")
        .expect("Failed to compile version replace regex 1")
});

// Pattern 2: name := version // comment or name := version # comment
static VERSION_REPLACE_RE_2: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\w+\s*:=\s*)([0-9]*\.?[0-9]+(?:\.[0-9]+)*)(\s*(?://|#).*)")
        .expect("Failed to compile version replace regex 2")
});

// Pattern 3: name: version // comment or name: version # comment
static VERSION_REPLACE_RE_3: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\w+:\s*)([0-9]*\.?[0-9]+(?:\.[0-9]+)*)(\s*(?://|#).*)")
        .expect("Failed to compile version replace regex 3")
});

// Pattern 4: "name:version" // comment
static VERSION_REPLACE_RE_4: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"("(\w+):)([0-9]*\.?[0-9]+(?:\.[0-9]+)*)(")(\s*(?://|#).*)"#)
        .expect("Failed to compile version replace regex 4")
});

// Pattern 5: "name": "version" // comment
static VERSION_REPLACE_RE_5: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"("(\w+)":\s*")([0-9]*\.?[0-9]+(?:\.[0-9]+)*)(")(\s*(?://|#).*)"#)
        .expect("Failed to compile version replace regex 5")
});

// Pattern 6: name = 'version' // comment (single quotes)
static VERSION_REPLACE_RE_6: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\w+\s*=\s*')([0-9]*\.?[0-9]+(?:\.[0-9]+)*)(')(\s*(?://|#).*)")
        .expect("Failed to compile version replace regex 6")
});

// Pattern 7: name := 'version' // comment (single quotes)
static VERSION_REPLACE_RE_7: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\w+\s*:=\s*')([0-9]*\.?[0-9]+(?:\.[0-9]+)*)(')(\s*(?://|#).*)")
        .expect("Failed to compile version replace regex 7")
});

// Pattern 8: name: 'version' // comment (single quotes)
static VERSION_REPLACE_RE_8: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\w+:\s*')([0-9]*\.?[0-9]+(?:\.[0-9]+)*)(')(\s*(?://|#).*)")
        .expect("Failed to compile version replace regex 8")
});

// Pattern 9: 'name:version' // comment (single quotes)
static VERSION_REPLACE_RE_9: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"('(\w+):)([0-9]*\.?[0-9]+(?:\.[0-9]+)*)(')(\s*(?://|#).*)")
        .expect("Failed to compile version replace regex 9")
});

// Pattern 10: 'name': 'version' // comment (single quotes)
static VERSION_REPLACE_RE_10: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"('(\w+)':\s*')([0-9]*\.?[0-9]+(?:\.[0-9]+)*)(')(\s*(?://|#).*)")
        .expect("Failed to compile version replace regex 10")
});

/// Array of all supported version patterns
pub static VERSION_PATTERNS: &[VersionPattern] = &[
    VersionPattern {
        replace_regex: &VERSION_REPLACE_RE_1,
        replacement_format: ReplacementFormat::Simple,
    },
    VersionPattern {
        replace_regex: &VERSION_REPLACE_RE_2,
        replacement_format: ReplacementFormat::Simple,
    },
    VersionPattern {
        replace_regex: &VERSION_REPLACE_RE_3,
        replacement_format: ReplacementFormat::Simple,
    },
    VersionPattern {
        replace_regex: &VERSION_REPLACE_RE_4,
        replacement_format: ReplacementFormat::Extended,
    },
    VersionPattern {
        replace_regex: &VERSION_REPLACE_RE_5,
        replacement_format: ReplacementFormat::Extended,
    },
    VersionPattern {
        replace_regex: &VERSION_REPLACE_RE_6,
        replacement_format: ReplacementFormat::Quoted,
    },
    VersionPattern {
        replace_regex: &VERSION_REPLACE_RE_7,
        replacement_format: ReplacementFormat::Quoted,
    },
    VersionPattern {
        replace_regex: &VERSION_REPLACE_RE_8,
        replacement_format: ReplacementFormat::Quoted,
    },
    VersionPattern {
        replace_regex: &VERSION_REPLACE_RE_9,
        replacement_format: ReplacementFormat::Extended,
    },
    VersionPattern {
        replace_regex: &VERSION_REPLACE_RE_10,
        replacement_format: ReplacementFormat::Extended,
    },
];

/// Attempts to replace the version in a line with a new version using all available patterns
///
/// # Arguments
/// * `line` - The line of text to replace version in
/// * `new_version` - The new version string to replace with
///
/// # Returns
/// * `Some(String)` containing the updated line if a pattern matches
/// * `None` if no pattern matches for replacement
pub fn try_replace_version_in_line(line: &str, new_version: &str) -> Option<String> {
    for pattern in VERSION_PATTERNS {
        if pattern.replace_regex.is_match(line) {
            return Some(
                pattern
                    .replace_regex
                    .replace_all(line, |caps: &regex::Captures| {
                        match pattern.replacement_format {
                            ReplacementFormat::Simple => {
                                format!("{}{}{}", &caps[1], new_version, &caps[3])
                            }
                            ReplacementFormat::Extended => {
                                format!("{}{}{}{}", &caps[1], new_version, &caps[4], &caps[5])
                            }
                            ReplacementFormat::Quoted => {
                                format!("{}{}{}{}", &caps[1], new_version, &caps[3], &caps[4])
                            }
                        }
                    })
                    .to_string(),
            );
        }
    }
    None
}

/// Cleans version tags by removing 'v' or 'V' prefixes
///
/// # Arguments
/// * `inp` - The input version string to clean
///
/// # Returns
/// * A cleaned version string with 'v' or 'V' prefixes removed
pub fn clean_tag(inp: String) -> String {
    if inp.starts_with(['V', 'v']) {
        inp.replace("v", "").replace("V", "")
    } else {
        inp
    }
}
