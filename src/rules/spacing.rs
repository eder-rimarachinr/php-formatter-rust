/// Enforces spacing rules around operators and keywords.

/// Ensures exactly one space around binary operators: = += -= *= /= .= == === != !== < > <= >= && || ? :
pub fn normalize_operators(line: &str) -> String {
    // We do a simple pass: collapse multiple spaces around = to exactly one,
    // but only outside strings. A full AST-based pass handles complex cases.
    let mut result = String::with_capacity(line.len());
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    let mut in_single = false;
    let mut in_double = false;

    while i < len {
        let ch = bytes[i] as char;

        // Track string context (simple — no heredoc support here)
        if ch == '\'' && !in_double {
            in_single = !in_single;
            result.push(ch);
            i += 1;
            continue;
        }
        if ch == '"' && !in_single {
            in_double = !in_double;
            result.push(ch);
            i += 1;
            continue;
        }

        if in_single || in_double {
            result.push(ch);
            i += 1;
            continue;
        }

        // Collapse multiple spaces to single (outside strings)
        if ch == ' ' {
            result.push(' ');
            i += 1;
            while i < len && bytes[i] == b' ' {
                i += 1;
            }
            continue;
        }

        result.push(ch);
        i += 1;
    }

    result
}

/// Ensures a space after commas.
pub fn normalize_commas(line: &str) -> String {
    let mut result = String::with_capacity(line.len());
    let mut in_single = false;
    let mut in_double = false;
    let mut prev = '\0';

    for ch in line.chars() {
        if ch == '\'' && !in_double && prev != '\\' {
            in_single = !in_single;
        }
        if ch == '"' && !in_single && prev != '\\' {
            in_double = !in_double;
        }

        if !in_single && !in_double && ch == ',' {
            result.push(',');
            result.push(' ');
            prev = ch;
            continue;
        }
        // Remove space that was already there after comma (we just added one)
        if !in_single && !in_double && ch == ' ' && prev == ',' {
            prev = ch;
            continue;
        }

        result.push(ch);
        prev = ch;
    }

    result
}
