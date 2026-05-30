use crate::rules::{indent, spacing};

/// Metadata for a single source line, used by the alignment pass.
#[derive(Debug, Default, Clone)]
pub struct Line {
    pub content: String,
    /// Byte offset of the `=` in this line (None if not a simple assignment).
    pub assignment_col: Option<usize>,
    /// Length of the LHS before `=` (without leading indent).
    pub lhs_len: usize,
    /// Column at which `=` should be placed after alignment (set by align pass).
    pub align_col: Option<usize>,
    /// Leading indent (spaces) already on the line.
    pub indent: usize,
}

impl Line {
    /// Render the final line, applying `align_col` if present.
    pub fn render(&self) -> String {
        let Some(target_col) = self.align_col else {
            return self.content.clone();
        };
        let Some(eq_pos) = self.assignment_col else {
            return self.content.clone();
        };

        let indent_str = " ".repeat(self.indent);
        let trimmed = self.content.trim_start();

        // Split at the `=` position relative to trimmed content
        let eq_in_trimmed = eq_pos.saturating_sub(self.indent);

        if eq_in_trimmed >= trimmed.len() {
            return self.content.clone();
        }

        let lhs = trimmed[..eq_in_trimmed].trim_end();
        let rhs = trimmed[eq_in_trimmed + 1..].trim_start();

        let padding = if target_col > lhs.len() {
            target_col - lhs.len()
        } else {
            1
        };

        format!("{}{}{} = {}", indent_str, lhs, " ".repeat(padding), rhs)
    }
}

pub struct Formatter {
    pub indent_size: usize,
}

impl Formatter {
    pub fn new(indent_size: usize) -> Self {
        Self { indent_size }
    }

    /// Main entry point: format a complete PHP source string.
    pub fn format(&self, source: &str) -> String {
        let raw_lines: Vec<&str> = source.lines().collect();
        let mut lines: Vec<Line> = raw_lines
            .iter()
            .map(|raw| self.process_line(raw))
            .collect();

        // Alignment pass: find consecutive assignment blocks and align `=`
        crate::rules::align::align_assignments(&mut lines);

        let mut out = lines.iter().map(|l| l.render()).collect::<Vec<_>>().join("\n");

        // Preserve a trailing newline if the original had one
        if source.ends_with('\n') {
            out.push('\n');
        }
        out
    }

    fn process_line(&self, raw: &str) -> Line {
        let normalized = indent::normalize(raw, self.indent_size);
        let normalized = spacing::normalize_operators(&normalized);
        let normalized = spacing::normalize_commas(&normalized);

        let indent_count = count_leading_spaces(&normalized);
        let (assignment_col, lhs_len) = detect_assignment(&normalized, indent_count);

        Line {
            content: normalized,
            assignment_col,
            lhs_len,
            align_col: None,
            indent: indent_count,
        }
    }
}

fn count_leading_spaces(s: &str) -> usize {
    s.chars().take_while(|c| *c == ' ').count()
}

/// Detect a simple assignment `$lhs = rhs` or `$lhs op= rhs`.
/// Returns (position of `=` in line, lhs trimmed length) or (None, 0).
fn detect_assignment(line: &str, indent: usize) -> (Option<usize>, usize) {
    let trimmed = line.trim();
    if trimmed.is_empty() || !trimmed.starts_with('$') {
        return (None, 0);
    }

    // We look for `=` that is NOT part of `==`, `!=`, `<=`, `>=`, `===`, `!==`
    let bytes = trimmed.as_bytes();
    let len = bytes.len();
    let mut in_single = false;
    let mut in_double = false;
    let mut i = 0;

    while i < len {
        let ch = bytes[i] as char;

        if ch == '\'' && !in_double {
            in_single = !in_single;
            i += 1;
            continue;
        }
        if ch == '"' && !in_single {
            in_double = !in_double;
            i += 1;
            continue;
        }
        if in_single || in_double {
            i += 1;
            continue;
        }

        if ch == '=' {
            // Skip ==, ===
            let next = if i + 1 < len { bytes[i + 1] } else { 0 };
            if next == b'=' {
                i += 2;
                continue;
            }
            // Skip !=, <=, >=, +=, -=, *=, /=, .=  — only those preceded by operator chars
            let prev = if i > 0 { bytes[i - 1] } else { 0 };
            if matches!(prev, b'!' | b'<' | b'>' | b'+' | b'-' | b'*' | b'/' | b'.') {
                i += 1;
                continue;
            }

            // This is a plain `=` assignment
            let lhs = trimmed[..i].trim();
            let eq_abs = indent + i;
            return (Some(eq_abs), lhs.len());
        }

        i += 1;
    }

    (None, 0)
}
