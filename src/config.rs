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
    /// Walk up from `from` searching for `.phpfmt.toml`. Falls back to defaults.
    pub fn discover(from: &Path) -> Self {
        let start = if from.is_file() {
            from.parent().unwrap_or(from)
        } else {
            from
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
            match dir.parent() {
                Some(p) if p != dir => dir = p,
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
