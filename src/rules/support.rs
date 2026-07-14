//! Small parsing helpers shared by rules.

/// If `line` is an ATX heading, return its level (1–6): 1–6 `#` characters
/// followed by a space or the end of the line. `#tag` (no space) is not a
/// heading.
pub fn heading_level(line: &str) -> Option<usize> {
    let trimmed = line.trim_start_matches([' ', '\t']);
    let hashes = trimmed.bytes().take_while(|&byte| byte == b'#').count();
    if !(1..=6).contains(&hashes) {
        return None;
    }
    let rest = &trimmed[hashes..];
    if rest.is_empty() || rest.starts_with(' ') || rest.starts_with('\t') {
        Some(hashes)
    } else {
        None
    }
}

/// Whether every whitespace-separated token on `line` is a Bear tag (so the
/// line is metadata, not prose).
pub fn is_pure_tag_line(line: &str) -> bool {
    let mut any = false;
    for token in line.split_whitespace() {
        any = true;
        let Some(inner) = token.strip_prefix('#') else {
            return false;
        };
        let core = inner.strip_suffix('#').unwrap_or(inner);
        if core.is_empty() || core.contains('#') {
            return false;
        }
    }
    any
}

/// The indentation width (in leading whitespace characters) if `line` is a list
/// item (bullet or ordered), else `None`.
pub fn list_item_indent(line: &str) -> Option<usize> {
    let stripped = line.trim_start_matches([' ', '\t']);
    let indent_width = line.len() - stripped.len();
    if starts_list_marker(stripped) {
        Some(indent_width)
    } else {
        None
    }
}

fn starts_list_marker(rest: &str) -> bool {
    if let Some(first) = rest.chars().next() {
        if ['-', '*', '+', '•', '§'].contains(&first) {
            let after = &rest[first.len_utf8()..];
            return after.is_empty() || after.starts_with(' ');
        }
    }
    let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
    if digits > 0 {
        if let Some(delimiter) = rest.as_bytes().get(digits) {
            if matches!(delimiter, b'.' | b')') {
                let after = &rest[digits + 1..];
                return after.is_empty() || after.starts_with(' ');
            }
        }
    }
    false
}
