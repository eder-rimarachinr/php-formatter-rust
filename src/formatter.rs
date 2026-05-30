use crate::config::Config;
use crate::rules::{align, align_arrows, align_comments, indent, spacing};

#[derive(Debug, Default, Clone)]
pub struct Line {
    /// Code portion of the line (before any inline comment), with trailing whitespace stripped.
    pub code_part: String,
    /// Inline comment suffix (`// ...` or `# ...`) — empty when absent.
    pub comment_part: String,

    // = alignment
    pub assignment_col: Option<usize>,
    pub lhs_len: usize,
    pub align_col: Option<usize>,

    // => alignment
    pub fat_arrow_col: Option<usize>,
    pub fat_arrow_lhs_len: usize,
    pub fat_arrow_align_col: Option<usize>,

    // comment alignment (set after = / => passes)
    pub comment_align_col: Option<usize>,

    pub indent: usize,
}

impl Line {
    /// Length of the rendered code part (with alignment applied).
    /// Called by the comment-alignment pass after = and => passes have run.
    pub fn rendered_code_len(&self) -> usize {
        self.render_code().trim_end().len()
    }

    pub fn render(&self) -> String {
        let code = self.render_code();

        if self.comment_part.is_empty() {
            return code;
        }

        let code_trimmed = code.trim_end();
        let pad = if let Some(target) = self.comment_align_col {
            target.saturating_sub(code_trimmed.len()).max(1)
        } else {
            1
        };
        format!("{}{}{}", code_trimmed, " ".repeat(pad), self.comment_part)
    }

    fn render_code(&self) -> String {
        if let Some(target_col) = self.align_col {
            return self.render_assignment(target_col);
        }
        if let Some(target_col) = self.fat_arrow_align_col {
            return self.render_fat_arrow(target_col);
        }
        self.code_part.clone()
    }

    fn render_assignment(&self, target_col: usize) -> String {
        let eq_pos = match self.assignment_col {
            Some(p) => p,
            None => return self.code_part.clone(),
        };
        let indent_str = " ".repeat(self.indent);
        let trimmed = self.code_part.trim_start();
        let eq_in_trimmed = eq_pos.saturating_sub(self.indent);
        if eq_in_trimmed >= trimmed.len() {
            return self.code_part.clone();
        }
        let lhs = trimmed[..eq_in_trimmed].trim_end();
        let rhs = trimmed[eq_in_trimmed + 1..].trim_start();
        let padding = target_col.saturating_sub(lhs.len()).max(1);
        format!("{}{}{} = {}", indent_str, lhs, " ".repeat(padding), rhs)
    }

    fn render_fat_arrow(&self, target_col: usize) -> String {
        let arrow_pos = match self.fat_arrow_col {
            Some(p) => p,
            None => return self.code_part.clone(),
        };
        let indent_str = " ".repeat(self.indent);
        let trimmed = self.code_part.trim_start();
        let arrow_in_trimmed = arrow_pos.saturating_sub(self.indent);
        // arrow_in_trimmed is the offset of `=` in `=>` within trimmed
        if arrow_in_trimmed + 1 >= trimmed.len() {
            return self.code_part.clone();
        }
        let lhs = trimmed[..arrow_in_trimmed].trim_end();
        let rhs = trimmed[arrow_in_trimmed + 2..].trim_start(); // skip `=>`
        let padding = target_col.saturating_sub(lhs.len()).max(1);
        format!("{}{}{} => {}", indent_str, lhs, " ".repeat(padding), rhs)
    }
}

pub struct Formatter {
    pub config: Config,
}

impl Formatter {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Format a complete PHP source string, respecting frozen regions.
    pub fn format(&self, source: &str) -> String {
        let regions = crate::rules::frozen::split_regions(source);
        let mut out = String::new();

        for region in &regions {
            if region.frozen {
                out.push_str(&region.content);
            } else {
                out.push_str(&self.format_region(&region.content));
            }
        }

        // Normalise trailing newline to match the original
        let had_newline = source.ends_with('\n');
        while out.ends_with('\n') {
            out.pop();
        }
        if had_newline {
            out.push('\n');
        }
        out
    }

    fn format_region(&self, source: &str) -> String {
        let raw_lines: Vec<&str> = source.lines().collect();
        if raw_lines.is_empty() {
            return String::new();
        }

        let mut lines: Vec<Line> = raw_lines
            .iter()
            .map(|raw| self.process_line(raw))
            .collect();

        if self.config.align.assignments {
            align::align_assignments(&mut lines);
        }
        if self.config.align.fat_arrows {
            align_arrows::align_fat_arrows(&mut lines);
        }
        if self.config.align.inline_comments {
            // Must run last — uses rendered lengths from the passes above
            align_comments::align_inline_comments(&mut lines);
        }

        let mut result = lines
            .iter()
            .map(|l| l.render())
            .collect::<Vec<_>>()
            .join("\n");
        result.push('\n');
        result
    }

    fn process_line(&self, raw: &str) -> Line {
        let normalized = indent::normalize(raw, self.config.indent_size);
        let normalized = spacing::normalize_operators(&normalized);
        let normalized = spacing::normalize_commas(&normalized);

        let (code_part, comment_part) = split_inline_comment(&normalized);
        let indent_count = count_leading_spaces(&code_part);

        let (assignment_col, lhs_len) = detect_assignment(&code_part, indent_count);

        // Only detect fat arrows on lines that aren't plain assignments
        let (fat_arrow_col, fat_arrow_lhs_len) = if assignment_col.is_none() {
            detect_fat_arrow(&code_part, indent_count)
        } else {
            (None, 0)
        };

        Line {
            code_part,
            comment_part,
            assignment_col,
            lhs_len,
            align_col: None,
            fat_arrow_col,
            fat_arrow_lhs_len,
            fat_arrow_align_col: None,
            comment_align_col: None,
            indent: indent_count,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn count_leading_spaces(s: &str) -> usize {
    s.chars().take_while(|c| *c == ' ').count()
}

/// Split a normalized line at the first unquoted `//` or `#`.
/// Returns (code_part, comment_part). comment_part is empty when absent.
fn split_inline_comment(line: &str) -> (String, String) {
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut in_single = false;
    let mut in_double = false;
    let mut i = 0;

    while i < len {
        let ch = bytes[i];

        if ch == b'\'' && !in_double {
            in_single = !in_single;
            i += 1;
            continue;
        }
        if ch == b'"' && !in_single {
            in_double = !in_double;
            i += 1;
            continue;
        }
        if in_single || in_double {
            i += 1;
            continue;
        }

        if ch == b'/' && i + 1 < len && bytes[i + 1] == b'/' {
            return (line[..i].trim_end().to_string(), line[i..].to_string());
        }
        if ch == b'#' {
            return (line[..i].trim_end().to_string(), line[i..].to_string());
        }

        i += 1;
    }

    (line.to_string(), String::new())
}

/// Detect a plain `$var = …` assignment.
/// Returns (byte position of `=` in line, LHS trimmed length) or (None, 0).
fn detect_assignment(line: &str, indent: usize) -> (Option<usize>, usize) {
    let trimmed = line.trim();
    if trimmed.is_empty() || !trimmed.starts_with('$') {
        return (None, 0);
    }

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
            let next = if i + 1 < len { bytes[i + 1] } else { 0 };
            // Skip ==, ===
            if next == b'=' {
                i += 2;
                continue;
            }
            // Skip => (fat arrow)
            if next == b'>' {
                i += 2;
                continue;
            }
            // Skip compound operators: !=, <=, >=, +=, -=, *=, /=, .=, %=, **=
            let prev = if i > 0 { bytes[i - 1] } else { 0 };
            if matches!(prev, b'!' | b'<' | b'>' | b'+' | b'-' | b'*' | b'/' | b'.' | b'%') {
                i += 1;
                continue;
            }

            let lhs = trimmed[..i].trim();
            return (Some(indent + i), lhs.len());
        }

        i += 1;
    }

    (None, 0)
}

/// Detect a fat-arrow `key => value` pattern.
/// Returns (byte position of `=` in `=>` within line, LHS trimmed length) or (None, 0).
fn detect_fat_arrow(line: &str, indent: usize) -> (Option<usize>, usize) {
    let trimmed = line.trim_start();
    if trimmed.is_empty() {
        return (None, 0);
    }

    let bytes = trimmed.as_bytes();
    let len = bytes.len();
    let mut in_single = false;
    let mut in_double = false;
    let mut i = 0;

    while i < len {
        let ch = bytes[i];

        if ch == b'\'' && !in_double {
            in_single = !in_single;
            i += 1;
            continue;
        }
        if ch == b'"' && !in_single {
            in_double = !in_double;
            i += 1;
            continue;
        }
        if in_single || in_double {
            i += 1;
            continue;
        }

        if ch == b'=' && i + 1 < len && bytes[i + 1] == b'>' {
            let lhs = trimmed[..i].trim_end();
            return (Some(indent + i), lhs.len());
        }

        i += 1;
    }

    (None, 0)
}
