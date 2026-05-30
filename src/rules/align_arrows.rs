use crate::formatter::Line;

/// Aligns `=>` operators in consecutive lines (array/match context).
pub fn align_fat_arrows(lines: &mut [Line]) {
    let mut i = 0;
    while i < lines.len() {
        if lines[i].fat_arrow_col.is_none() {
            i += 1;
            continue;
        }

        let block_start = i;
        while i < lines.len() && lines[i].fat_arrow_col.is_some() {
            i += 1;
        }
        let block_end = i;

        if block_end - block_start < 2 {
            continue;
        }

        let max_lhs = lines[block_start..block_end]
            .iter()
            .map(|l| l.fat_arrow_lhs_len)
            .max()
            .unwrap_or(0);

        for line in &mut lines[block_start..block_end] {
            line.fat_arrow_align_col = Some(max_lhs + 1);
        }
    }
}
