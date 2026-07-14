//! **footnotes** — renumber footnotes by order of first reference and move all
//! definitions to the bottom. Handles single-line definitions (`[^label]:
//! text`); multi-line definitions are left where they are.

use std::collections::HashMap;

use crate::engine::ignore::IgnoreRanges;
use crate::engine::Rule;

pub struct Footnotes;

impl Rule for Footnotes {
    fn name(&self) -> &'static str {
        "footnotes"
    }

    fn apply(&self, text: &str, ignore: &IgnoreRanges) -> String {
        let mut definitions: HashMap<String, String> = HashMap::new();
        let mut body: Vec<String> = Vec::new();
        let mut start = 0;
        for piece in text.split_inclusive('\n') {
            let content = piece.strip_suffix('\n').unwrap_or(piece);
            let is_definition = !ignore.contains(start)
                && parse_definition(content)
                    .map(|(label, def)| definitions.insert(label, def))
                    .is_some();
            if !is_definition {
                body.push(content.to_string());
            }
            start += piece.len();
        }

        let numbers = assign_numbers(&body, &definitions);
        if numbers.is_empty() {
            return text.to_string();
        }

        let mut lines: Vec<String> = body
            .iter()
            .map(|line| renumber_references(line, &numbers))
            .collect();

        let mut defs: Vec<(usize, String)> = definitions
            .iter()
            .filter_map(|(label, def)| numbers.get(label).map(|number| (*number, def.clone())))
            .collect();
        defs.sort_by_key(|(number, _)| *number);

        while lines.last().is_some_and(|line| line.trim().is_empty()) {
            lines.pop();
        }
        if !defs.is_empty() {
            lines.push(String::new());
            for (number, def) in defs {
                lines.push(format!("[^{number}]: {def}"));
            }
        }

        let joined = lines.join("\n");
        if text.ends_with('\n') {
            format!("{joined}\n")
        } else {
            joined
        }
    }
}

/// Assign each footnote label a number: referenced labels first (in order of
/// first appearance), then any defined-but-unreferenced labels, alphabetically.
fn assign_numbers(
    body: &[String],
    definitions: &HashMap<String, String>,
) -> HashMap<String, usize> {
    let mut numbers = HashMap::new();
    let mut next = 1;
    for line in body {
        for label in references(line) {
            numbers.entry(label).or_insert_with(|| {
                let number = next;
                next += 1;
                number
            });
        }
    }
    let mut orphans: Vec<&String> = definitions
        .keys()
        .filter(|label| !numbers.contains_key(*label))
        .collect();
    orphans.sort();
    for label in orphans {
        numbers.insert(label.clone(), next);
        next += 1;
    }
    numbers
}

fn parse_definition(line: &str) -> Option<(String, String)> {
    let rest = line.trim_start().strip_prefix("[^")?;
    let close = rest.find("]:")?;
    let label = &rest[..close];
    if label.is_empty() || label.contains(char::is_whitespace) {
        return None;
    }
    Some((label.to_string(), rest[close + 2..].trim().to_string()))
}

fn references(line: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = line;
    while let Some(open) = rest.find("[^") {
        let after = &rest[open + 2..];
        let Some(close) = after.find(']') else {
            break;
        };
        let label = &after[..close];
        if !label.is_empty() && !label.contains(char::is_whitespace) {
            found.push(label.to_string());
        }
        rest = &after[close + 1..];
    }
    found
}

fn renumber_references(line: &str, numbers: &HashMap<String, usize>) -> String {
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(open) = rest.find("[^") {
        out.push_str(&rest[..open]);
        let after = &rest[open + 2..];
        let Some(close) = after.find(']') else {
            out.push_str("[^");
            rest = after;
            continue;
        };
        let label = &after[..close];
        match numbers.get(label) {
            Some(number) => out.push_str(&format!("[^{number}]")),
            None => out.push_str(&format!("[^{label}]")),
        }
        rest = &after[close + 1..];
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::Footnotes;
    use crate::engine::ignore::IgnoreRanges;
    use crate::engine::Rule;

    fn apply(text: &str) -> String {
        Footnotes.apply(text, &IgnoreRanges::compute(text))
    }

    #[test]
    fn renumbers_and_moves_definitions_to_bottom() {
        let input = "See b[^b] and a[^a].\n[^a]: first\n[^b]: second";
        assert_eq!(
            apply(input),
            "See b[^1] and a[^2].\n\n[^1]: second\n[^2]: first"
        );
    }

    #[test]
    fn leaves_footnoteless_text_alone() {
        assert_eq!(apply("no footnotes here"), "no footnotes here");
    }

    #[test]
    fn is_idempotent() {
        let once = apply("x[^x] y[^y]\n[^y]: yes\n[^x]: ex");
        assert_eq!(apply(&once), once);
    }
}
