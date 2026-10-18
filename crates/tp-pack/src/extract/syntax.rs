//! The text subset used by SII definitions.

use std::collections::BTreeMap;

#[derive(Debug, Default, PartialEq)]
pub struct Section {
    pub header: String,
    pub fields: BTreeMap<String, Vec<String>>,
    pub children: Vec<Section>,
}

impl Section {
    pub fn field(&self, name: &str) -> Result<&str, String> {
        let values = self
            .fields
            .get(name)
            .ok_or_else(|| format!("missing {name}"))?;
        let [value] = values.as_slice() else {
            return Err(format!("ambiguous {name}"));
        };
        Ok(unquote(value))
    }
}

/// Strips one pair of surrounding quotes.
pub fn unquote(value: &str) -> &str {
    match value.strip_prefix('"').and_then(|v| v.strip_suffix('"')) {
        Some(inner) if !inner.contains('"') => inner,
        _ => value,
    }
}

/// Quotes and parentheses shield delimiters; comments are omitted outside quotes.
pub fn parse(text: &str) -> Result<Section, String> {
    let mut stack = vec![Section::default()];
    let mut statement = String::new();
    let mut chars = text.chars().peekable();
    let mut quoted = false;
    let mut escaped = false;
    let mut parentheses = 0usize;
    while let Some(c) = chars.next() {
        if quoted {
            statement.push(c);
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                quoted = false;
            }
            continue;
        }
        match c {
            '"' => {
                quoted = true;
                statement.push(c);
            }
            '#' | '/' if c == '#' || chars.peek() == Some(&'/') => {
                // Skips the rest of the line, including a second '/'. The line then
                // ends as at a newline: a unit header may be followed by a comment.
                chars.by_ref().find(|&next| next == '\n');
                if parentheses == 0 && chars.clone().find(|next| !next.is_whitespace()) == Some('{')
                {
                    statement.push(' ');
                } else {
                    finish(&mut statement, stack.last_mut().unwrap())?;
                }
            }
            '(' => {
                parentheses += 1;
                statement.push(c);
            }
            ')' => {
                parentheses = parentheses.checked_sub(1).ok_or("unmatched parenthesis")?;
                statement.push(c);
            }
            '{' if parentheses == 0 => {
                let header = statement.trim().to_owned();
                if header.is_empty() {
                    return Err("section without header".into());
                }
                statement.clear();
                stack.push(Section {
                    header,
                    ..Default::default()
                });
            }
            '}' if parentheses == 0 => {
                finish(&mut statement, stack.last_mut().unwrap())?;
                if stack.len() == 1 {
                    return Err("unmatched closing section".into());
                }
                let child = stack.pop().unwrap();
                stack.last_mut().unwrap().children.push(child);
            }
            '\n' if parentheses == 0 => {
                // SII unit headers can put the opening brace on the next line.
                if chars.clone().find(|next| !next.is_whitespace()) == Some('{') {
                    statement.push(' ');
                } else {
                    finish(&mut statement, stack.last_mut().unwrap())?;
                }
            }
            _ => statement.push(c),
        }
    }
    if quoted || parentheses != 0 || stack.len() != 1 {
        return Err("unclosed quote, tuple or section".into());
    }
    finish(&mut statement, stack.last_mut().unwrap())?;
    Ok(stack.pop().unwrap())
}

fn finish(statement: &mut String, section: &mut Section) -> Result<(), String> {
    let line = statement.trim();
    if let Some((key, value)) = line.split_once(':') {
        let key = key.trim();
        if key.is_empty() || key.chars().any(char::is_whitespace) {
            return Err(format!("invalid field: {line}"));
        }
        let entries = section.fields.entry(key.to_owned()).or_default();
        entries.push(value.trim().to_owned());
    }
    statement.clear();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_unit_header_can_end_with_a_comment() {
        let text = "SiiNunit\n{\naccessory_addon_data : bpl100.example.chs_r_l #(100cm)\n{\nname: box\n}\n}\n";
        let unit = &parse(text).unwrap().children[0].children[0];
        assert_eq!(unit.header, "accessory_addon_data : bpl100.example.chs_r_l");
        assert_eq!(unit.field("name").unwrap(), "box");
    }

    #[test]
    fn sections_keep_empty_fields_quotes_and_arrays() {
        let text = "SiiNunit\n{\naccessory_cabin_data : high.example.cabin\n{\nname: \"Cab #1: {high}\" // comment\nmodel: \"/shared/model.pmd\"\nsuitable_for[]: one\nsuitable_for[]: two\n}\n}\n";
        let tree = parse(text).unwrap();
        let unit = &tree.children[0].children[0];
        assert_eq!(unit.field("name").unwrap(), "Cab #1: {high}");
        assert_eq!(unit.fields["suitable_for[]"], ["one", "two"]);
        let tree = parse("Part {\nPieces: \nLocators: 0 1\n}\n").unwrap();
        assert_eq!(tree.children[0].field("Pieces").unwrap(), "");
        assert_eq!(tree.children[0].field("Locators").unwrap(), "0 1");
    }

    #[test]
    fn malformed_or_conflicting_fields_fail() {
        for text in ["Piece {\nIndex: 0", "}", "Field: ( 1"] {
            assert!(parse(text).is_err(), "accepted {text}");
        }
        assert!(
            parse("Part {\nName: one\nName: two\n}").unwrap().children[0]
                .field("Name")
                .is_err()
        );
    }
}
