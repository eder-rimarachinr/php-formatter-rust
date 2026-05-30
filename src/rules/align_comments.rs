use crate::formatter::Line;

/// Aligns inline `//` or `#` comments in consecutive lines that all have one.
/// Must run AFTER assignment and fat-arrow passes so `rendered_code_len` is accurate.
pub fn align_inline_comments(lines: &mut [Line]) {
    let mut i = 0;
    while i < lines.len() {
        if lines[i].comment_part.is_empty() {
            i += 1;
            continue;
        }

        let block_start = i;
        while i < lines.len() && !lines[i].comment_part.is_empty() {
            i += 1;
        }
        let block_end = i;

        if block_end - block_start < 2 {
            continue;
        }

        // Use rendered code length (reflects = and => alignment already applied)
        let max_code_len = lines[block_start..block_end]
            .iter()
            .map(|l| l.rendered_code_len())
            .max()
            .unwrap_or(0);

        let target = max_code_len + 2;
        for line in &mut lines[block_start..block_end] {
            line.comment_align_col = Some(target);
        }
    }
}
