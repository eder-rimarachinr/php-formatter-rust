use serde::Deserialize;
use std::path::Path;

// ─── Raw serde types (TOML shape) ─────────────────────────────────────────────
//
// FIX-3: #[serde(deny_unknown_fields)] causes toml::from_str to return Err for
// any key not listed here — unknown fields can no longer silently influence
// formatter behaviour or be used to probe future options.

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct TomlRoot {
    #[serde(default)]
    style: TomlStyle,
    #[serde(default)]
    align: TomlAlign,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct TomlStyle {
    indent_size: Option<usize>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct TomlAlign {
    assignments:     Option<bool>,
    fat_arrows:      Option<bool>,
    inline_comments: Option<bool>,
}

// ─── Domain types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AlignConfig {
    pub assignments:     bool,
    pub fat_arrows:      bool,
    pub inline_comments: bool,
}

impl Default for AlignConfig {
    fn default() -> Self {
        Self { assignments: true, fat_arrows: true, inline_comments: true }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub indent_size: usize,
    pub align: AlignConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self { indent_size: 4, align: AlignConfig::default() }
    }
}

// ─── ConfigError ──────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq)]
pub enum ConfigError {
    IndentSizeOutOfRange(usize),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::IndentSizeOutOfRange(n) => {
                write!(f, "indent_size must be 1–16, got {n}")
            }
        }
    }
}

// ─── Config impl ──────────────────────────────────────────────────────────────

impl Config {
    /// FIX-3: validates all field values are within permitted ranges.
    /// Call after construction from user-supplied input.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if !(1..=16).contains(&self.indent_size) {
            return Err(ConfigError::IndentSizeOutOfRange(self.indent_size));
        }
        Ok(())
    }

    /// Walk up from `from` searching for `.phpfmt.toml`, stopping at `stop_at`.
    /// Both paths are canonicalized first to resolve symlinks (SEC-007).
    /// Discovery never traverses past `stop_at` (SEC-006).
    pub fn discover(from: &Path, stop_at: Option<&Path>) -> Self {
        let canonical_from =
            std::fs::canonicalize(from).unwrap_or_else(|_| from.to_path_buf());
        let canonical_stop = stop_at.map(|p| {
            std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf())
        });

        let start = if canonical_from.is_file() {
            canonical_from.parent().unwrap_or(&canonical_from).to_path_buf()
        } else {
            canonical_from.clone()
        };

        let mut dir = start;
        loop {
            let candidate = dir.join(".phpfmt.toml");
            if candidate.exists() {
                if let Ok(content) = std::fs::read_to_string(&candidate) {
                    if let Some(cfg) = Self::from_toml_str(&content) {
                        return cfg;
                    }
                }
                // File exists but is invalid → stop here, don't search higher.
                break;
            }
            if let Some(ref root) = canonical_stop {
                if dir == *root { break; }
            }
            match dir.parent() {
                Some(p) if p != dir => dir = p.to_path_buf(),
                _ => break,
            }
        }

        Self::default()
    }

    /// FIX-3: parse `.phpfmt.toml` via serde.
    /// Returns `None` when the TOML is malformed OR contains unknown fields OR
    /// any value fails `validate()` — caller falls back to `Config::default()`.
    fn from_toml_str(content: &str) -> Option<Self> {
        let raw: TomlRoot = toml::from_str(content).ok()?;

        let mut cfg = Self::default();
        if let Some(n) = raw.style.indent_size     { cfg.indent_size          = n; }
        if let Some(b) = raw.align.assignments     { cfg.align.assignments     = b; }
        if let Some(b) = raw.align.fat_arrows      { cfg.align.fat_arrows      = b; }
        if let Some(b) = raw.align.inline_comments { cfg.align.inline_comments = b; }

        // FIX-3: reject configs with out-of-range values rather than silently clamping.
        cfg.validate().ok()?;
        Some(cfg)
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── from_toml_str ──────────────────────────────────────────────────────────

    #[test]
    fn empty_toml_returns_default() {
        let cfg = Config::from_toml_str("").expect("empty TOML should yield defaults");
        assert_eq!(cfg.indent_size, 4);
        assert!(cfg.align.assignments);
    }

    #[test]
    fn valid_style_section_parses() {
        let cfg = Config::from_toml_str("[style]\nindent_size = 2")
            .expect("valid config should parse");
        assert_eq!(cfg.indent_size, 2);
    }

    #[test]
    fn valid_align_section_parses() {
        let cfg = Config::from_toml_str("[align]\nassignments = false\nfat_arrows = false")
            .expect("valid config should parse");
        assert!(!cfg.align.assignments);
        assert!(!cfg.align.fat_arrows);
    }

    #[test]
    fn unknown_key_in_style_returns_none() {
        // FIX-3: deny_unknown_fields rejects this
        let result = Config::from_toml_str("[style]\nindent_size = 2\nunknown_key = true");
        assert!(result.is_none(), "unknown field must be rejected");
    }

    #[test]
    fn unknown_section_returns_none() {
        let result = Config::from_toml_str("[style]\nindent_size = 2\n[future]\nkey = 1");
        assert!(result.is_none(), "unknown top-level section must be rejected");
    }

    #[test]
    fn unknown_key_in_align_returns_none() {
        let result = Config::from_toml_str("[align]\nassignments = true\nstrict = true");
        assert!(result.is_none(), "unknown align field must be rejected");
    }

    #[test]
    fn malformed_toml_returns_none() {
        let result = Config::from_toml_str("not = [valid toml");
        assert!(result.is_none());
    }

    #[test]
    fn indent_size_out_of_range_high_returns_none() {
        // FIX-3: validate() rejects 17 even though it is parseable TOML
        let result = Config::from_toml_str("[style]\nindent_size = 17");
        assert!(result.is_none(), "indent_size = 17 must be rejected");
    }

    #[test]
    fn indent_size_zero_returns_none() {
        let result = Config::from_toml_str("[style]\nindent_size = 0");
        assert!(result.is_none(), "indent_size = 0 must be rejected");
    }

    // ── validate() ────────────────────────────────────────────────────────────

    #[test]
    fn validate_accepts_boundary_values() {
        for size in [1usize, 4, 8, 16] {
            let mut cfg = Config::default();
            cfg.indent_size = size;
            assert!(
                cfg.validate().is_ok(),
                "indent_size = {size} should be valid"
            );
        }
    }

    #[test]
    fn validate_rejects_zero() {
        let mut cfg = Config::default();
        cfg.indent_size = 0;
        assert_eq!(
            cfg.validate().unwrap_err(),
            ConfigError::IndentSizeOutOfRange(0)
        );
    }

    #[test]
    fn validate_rejects_above_sixteen() {
        let mut cfg = Config::default();
        cfg.indent_size = 99;
        assert_eq!(
            cfg.validate().unwrap_err(),
            ConfigError::IndentSizeOutOfRange(99)
        );
    }

    #[test]
    fn config_error_display_includes_value() {
        let msg = ConfigError::IndentSizeOutOfRange(42).to_string();
        assert!(msg.contains("42"));
    }
}
