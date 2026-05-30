use std::path::Path;

#[derive(Debug, Clone)]
pub struct AlignConfig {
    pub assignments: bool,
    pub fat_arrows: bool,
    pub inline_comments: bool,
}

impl Default for AlignConfig {
    fn default() -> Self {
        Self {
            assignments: true,
            fat_arrows: true,
            inline_comments: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub indent_size: usize,
    pub align: AlignConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            indent_size: 4,
            align: AlignConfig::default(),
        }
    }
}

impl Config {
    /// Walk up from `from` searching for `.phpfmt.toml`, stopping at `stop_at`.
    /// Both paths are canonicalized first to resolve symlinks (SEC-007).
    /// Discovery never traverses past `stop_at`, preventing config pickup from
    /// ancestor directories outside the workspace (SEC-006).
    pub fn discover(from: &Path, stop_at: Option<&Path>) -> Self {
        // SEC-007: canonicalize to resolve symlinks before traversal.
        let canonical_from = std::fs::canonicalize(from).unwrap_or_else(|_| from.to_path_buf());
        let canonical_stop = stop_at.map(|p| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf()));

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
                    return Self::from_toml(&content);
                }
                break;
            }
            // SEC-006: stop at workspace root — never read config from ancestors
            // outside the workspace boundary supplied by the extension.
            if let Some(ref root) = canonical_stop {
                if dir == *root {
                    break;
                }
            }
            match dir.parent() {
                Some(p) if p != dir => dir = p.to_path_buf(),
                _ => break,
            }
        }

        Self::default()
    }

    /// Minimal TOML parser for our config subset.
    /// Supports `[section]` headers and `key = value` pairs (bool/int).
    fn from_toml(content: &str) -> Self {
        let mut cfg = Self::default();
        let mut section = "";

        for raw in content.lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                // store with brackets so match arms are unambiguous
                section = line;
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                let key = k.trim();
                let val = v.trim().trim_matches('"').trim_matches('\'');
                match (section, key) {
                    ("[style]", "indent_size") => {
                        if let Ok(n) = val.parse::<usize>() {
                            cfg.indent_size = n.clamp(1, 16);
                        }
                    }
                    ("[align]", "assignments") => {
                        cfg.align.assignments = val == "true";
                    }
                    ("[align]", "fat_arrows") => {
                        cfg.align.fat_arrows = val == "true";
                    }
                    ("[align]", "inline_comments") => {
                        cfg.align.inline_comments = val == "true";
                    }
                    _ => {}
                }
            }
        }

        cfg
    }
}
