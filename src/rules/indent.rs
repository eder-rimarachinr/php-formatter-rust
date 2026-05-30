/// Normalizes indentation to 4 spaces per level.
/// Converts tabs to spaces and trims trailing whitespace.
pub fn normalize(line: &str, indent_size: usize) -> String {
    let trimmed = line.trim_end();
    if trimmed.is_empty() {
        return String::new();
    }
    // Count leading whitespace (tabs count as indent_size spaces)
    let mut depth = 0usize;
    let mut chars = trimmed.chars().peekable();
    loop {
        match chars.peek() {
            Some('\t') => {
                depth += indent_size;
                chars.next();
            }
            Some(' ') => {
                depth += 1;
                chars.next();
            }
            _ => break,
        }
    }
    let rest: String = chars.collect();
    let spaces = " ".repeat(depth);
    format!("{}{}", spaces, rest)
}

/// Re-indents a line to the given depth (in spaces).
pub fn reindent(line: &str, depth: usize, indent_size: usize) -> String {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let spaces = " ".repeat(depth * indent_size);
    format!("{}{}", spaces, trimmed)
}
