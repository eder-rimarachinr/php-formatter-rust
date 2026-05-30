/// Aligns `=` operators in consecutive assignment blocks.
///
/// Example input:
///   $a = 1;
///   $foo = 2;
///   $longerVar = 3;
///
/// Example output:
///   $a         = 1;
///   $foo       = 2;
///   $longerVar = 3;
use crate::formatter::Line;

pub fn align_assignments(lines: &mut Vec<Line>) {
    let mut i = 0;
    while i < lines.len() {
        // Find start of an assignment block
        if lines[i].assignment_col.is_none() {
            i += 1;
            continue;
        }
        // Collect consecutive assignment lines
        let block_start = i;
        while i < lines.len() && lines[i].assignment_col.is_some() {
            i += 1;
        }
        let block_end = i;
        if block_end - block_start < 2 {
            continue;
        }
        // Find the maximum LHS length (before `=`)
        let max_lhs = lines[block_start..block_end]
            .iter()
            .map(|l| l.lhs_len)
            .max()
            .unwrap_or(0);

        for line in &mut lines[block_start..block_end] {
            line.align_col = Some(max_lhs + 1); // +1 for one space before =
        }
    }
}
