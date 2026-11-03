//! CSV and TSV extraction.
//!
//! Each data row becomes one line of text: the header repeated as
//! `column = "value"` pairs. A search for a column name or a cell value then
//! lands on the row that holds it. Lines are separated by newlines, so a
//! newline inside a cell is flattened to a space.

use std::fs::File;
use std::io::Read;
use std::path::Path;

use super::text::decode_utf16_bom;

/// Only the start of the file is used to choose the delimiter.
const SNIFF_BYTES: usize = 8 * 1024;
/// How many records are compared when choosing the delimiter.
const SNIFF_RECORDS: usize = 20;

/// Extracts row text from a CSV or TSV file, reading at most `max_bytes`.
pub fn extract_delimited(path: &Path, max_bytes: usize) -> Option<String> {
    let mut file = File::open(path).ok()?;
    let truncated = file.metadata().ok()?.len() > max_bytes as u64;
    let mut buffer = Vec::new();
    (&mut file)
        .take(max_bytes as u64)
        .read_to_end(&mut buffer)
        .ok()?;
    let rows = row_blocks(&buffer, truncated, hint_for_path(path));
    join_within_cap(&rows, max_bytes)
}

/// Decodes `buffer` and returns one block of text per data row.
///
/// `truncated` means the buffer is only the start of a larger file, so its
/// last record may be cut off and is dropped. `hint` is the delimiter to
/// prefer when candidates look equally consistent (tab for `.tsv`).
pub(crate) fn row_blocks(buffer: &[u8], truncated: bool, hint: char) -> Vec<String> {
    let Some(text) = decode_text(buffer) else {
        return Vec::new();
    };
    if text.trim().is_empty() {
        return Vec::new();
    }

    let delimiter = sniff_delimiter(&text, hint);
    let mut records = parse_records(&text, delimiter);
    if truncated {
        records.pop();
    }
    let mut records = records.into_iter();
    let Some(header) = records.next() else {
        return Vec::new();
    };

    let names: Vec<String> = header
        .iter()
        .enumerate()
        .map(|(index, raw)| column_name(raw, index))
        .collect();

    let blocks: Vec<String> = records
        .map(|record| {
            record
                .iter()
                .enumerate()
                .map(|(index, value)| {
                    let name = names
                        .get(index)
                        .cloned()
                        .unwrap_or_else(|| fallback_name(index));
                    let value = one_line(value);
                    format!("{name} = \"{value}\"")
                })
                .collect::<Vec<_>>()
                .join(", ")
        })
        .collect();

    if blocks.is_empty() {
        // Header only: index the column names so they stay searchable.
        return vec![names.join(", ")];
    }
    blocks
}

/// Joins row blocks with newlines, keeping whole rows only. If the first row
/// alone is longer than `cap`, it is cut at a character boundary instead.
fn join_within_cap(blocks: &[String], cap: usize) -> Option<String> {
    let mut out = String::new();
    for block in blocks {
        let needed = if out.is_empty() {
            block.len()
        } else {
            out.len() + 1 + block.len()
        };
        if needed <= cap {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(block);
        } else {
            if out.is_empty() {
                out.push_str(clip_to_char_boundary(block, cap));
            }
            break;
        }
    }
    if out.trim().is_empty() {
        None
    } else {
        Some(out)
    }
}

/// Decodes UTF-16 (BOM), or UTF-8 with an optional UTF-8 BOM. Returns None for
/// binary data, using the same NUL-byte check as plain text.
fn decode_text(buffer: &[u8]) -> Option<String> {
    if let Some(text) = decode_utf16_bom(buffer) {
        return Some(text);
    }
    if buffer.iter().take(1024).any(|&byte| byte == 0) {
        return None;
    }
    let text = match std::str::from_utf8(buffer) {
        Ok(valid) => valid.to_string(),
        Err(_) => String::from_utf8_lossy(buffer).into_owned(),
    };
    Some(text.strip_prefix('\u{feff}').unwrap_or(text.as_str()).to_string())
}

/// Picks the delimiter whose first records agree on the most fields.
/// Ties go to the earlier candidate, so the hint is tried first.
fn sniff_delimiter(text: &str, hint: char) -> char {
    let sample = clip_to_char_boundary(text, SNIFF_BYTES);
    let candidates = [hint, ',', ';', '\t'];
    // (delimiter, records that agree on the width, field count)
    let mut best = (hint, 0, 0);
    for delimiter in candidates {
        let records: Vec<Vec<String>> = parse_records(sample, delimiter)
            .into_iter()
            .take(SNIFF_RECORDS)
            .collect();
        let Some(width) = records.first().map(Vec::len) else {
            continue;
        };
        if width < 2 {
            continue;
        }
        let agreeing = records.iter().filter(|record| record.len() == width).count();
        if (agreeing, width) > (best.1, best.2) {
            best = (delimiter, agreeing, width);
        }
    }
    best.0
}

/// Splits text into records of fields, following RFC 4180 quoting: a quoted
/// field may contain the delimiter, line breaks and doubled quotes. Blank
/// lines are skipped. An unclosed quote runs to the end of the text.
fn parse_records(text: &str, delimiter: char) -> Vec<Vec<String>> {
    let mut records = Vec::new();
    let mut record: Vec<String> = Vec::new();
    let mut field = String::new();
    let mut in_quotes = false;
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if in_quotes {
            if c != '"' {
                field.push(c);
            } else if chars.peek() == Some(&'"') {
                // A doubled quote inside quotes is one literal quote.
                field.push('"');
                chars.next();
            } else {
                in_quotes = false;
            }
        } else if c == '"' && field.is_empty() {
            in_quotes = true;
        } else if c == delimiter {
            record.push(std::mem::take(&mut field));
        } else if c == '\n' || c == '\r' {
            if c == '\r' && chars.peek() == Some(&'\n') {
                chars.next();
            }
            end_record(&mut record, &mut field, &mut records);
        } else {
            field.push(c);
        }
    }
    if !field.is_empty() || !record.is_empty() {
        end_record(&mut record, &mut field, &mut records);
    }
    records
}

fn end_record(record: &mut Vec<String>, field: &mut String, records: &mut Vec<Vec<String>>) {
    record.push(std::mem::take(field));
    // A blank line is a single empty field: skip it.
    if record.len() == 1 && record[0].is_empty() {
        record.clear();
        return;
    }
    records.push(std::mem::take(record));
}

fn column_name(raw: &str, index: usize) -> String {
    let name = raw.trim();
    if name.is_empty() {
        fallback_name(index)
    } else {
        name.to_string()
    }
}

fn fallback_name(index: usize) -> String {
    format!("column{}", index + 1)
}

/// Cell values are trimmed and any line breaks become spaces, so that a
/// newline always means "next row".
fn one_line(value: &str) -> String {
    value.trim().replace(['\n', '\r'], " ")
}

/// Delimiter to prefer on ties: tab for `.tsv`, comma otherwise.
pub(crate) fn hint_for_path(path: &Path) -> char {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case("tsv") => '\t',
        _ => ',',
    }
}

/// Longest prefix of `text` that is at most `max_bytes` long and ends on a
/// character boundary.
pub(crate) fn clip_to_char_boundary(text: &str, max_bytes: usize) -> &str {
    let mut end = max_bytes.min(text.len());
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    /// A temp file that removes itself on drop.
    struct Fixture(PathBuf);

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

    fn file_with(tag: &str, bytes: &[u8]) -> Fixture {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "kensaku-delimited-{tag}-{}-{nanos}.csv",
            std::process::id()
        ));
        std::fs::write(&path, bytes).unwrap();
        Fixture(path)
    }

    fn rows(text: &str) -> Vec<String> {
        row_blocks(text.as_bytes(), false, ',')
    }

    #[test]
    fn quoted_newline_stays_inside_one_row() {
        let text = "name,note\nAda,\"line one\nline two\"\n";
        assert_eq!(
            rows(text),
            vec!["name = \"Ada\", note = \"line one line two\""]
        );
    }

    #[test]
    fn semicolon_export_is_sniffed_and_keeps_decimal_commas() {
        assert_eq!(
            rows("name;amount\nAda;1,5\n"),
            vec!["name = \"Ada\", amount = \"1,5\""]
        );
    }

    #[test]
    fn tsv_file_uses_tab() {
        assert_eq!(
            row_blocks(b"a\tb\n1\t2\n", false, '\t'),
            vec!["a = \"1\", b = \"2\""]
        );
    }

    #[test]
    fn tab_hint_wins_a_tie_against_commas_in_values() {
        assert_eq!(
            row_blocks("a\tb,c\n1\t2,3\n".as_bytes(), false, '\t'),
            vec!["a = \"1\", b,c = \"2,3\""]
        );
    }

    #[test]
    fn utf8_bom_is_removed_before_the_header() {
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend_from_slice(b"name,city\nAda,London\n");
        assert_eq!(
            row_blocks(&bytes, false, ','),
            vec!["name = \"Ada\", city = \"London\""]
        );
    }

    #[test]
    fn utf16_le_file_extracts() {
        let mut bytes = vec![0xFF, 0xFE];
        bytes.extend(
            "name,city\nAda,London\n"
                .encode_utf16()
                .flat_map(u16::to_le_bytes),
        );
        let fixture = file_with("utf16", &bytes);
        assert_eq!(
            extract_delimited(&fixture.0, 10_000).as_deref(),
            Some("name = \"Ada\", city = \"London\"")
        );
    }

    #[test]
    fn empty_and_whitespace_only_files_are_none() {
        let empty = file_with("empty", b"");
        assert_eq!(extract_delimited(&empty.0, 1000), None);
        let blank = file_with("blank", b"  \n\n");
        assert_eq!(extract_delimited(&blank.0, 1000), None);
    }

    #[test]
    fn row_wider_than_the_cap_is_cut_not_dropped() {
        let text = format!("name\n{}\n", "x".repeat(300));
        let out = join_within_cap(&rows(&text), 40).unwrap();
        assert!(out.len() <= 40);
        assert!(out.starts_with("name = \"xxx"));
    }

    #[test]
    fn malformed_input_does_not_panic_or_hang() {
        for text in ["a,b\n1,\"oops\n2,3", "\"", "a,\"b\"c,d\n", ",,,\n\"\"\"\n"] {
            let _ = rows(text);
        }
        assert!(!rows("a,b\n1,\"oops\n2,3").is_empty());
    }

    #[test]
    fn ragged_rows_keep_every_value() {
        assert_eq!(
            rows("a,b,c\n1,2\n1,2,3,4\n"),
            vec![
                "a = \"1\", b = \"2\"",
                "a = \"1\", b = \"2\", c = \"3\", column4 = \"4\"",
            ]
        );
    }

    #[test]
    fn header_only_file_indexes_the_column_names() {
        assert_eq!(rows("name,city\n"), vec!["name, city"]);
    }

    #[test]
    fn truncated_input_drops_the_partial_last_row() {
        assert_eq!(
            row_blocks(b"a,b\n1,2\n3,4", true, ','),
            vec!["a = \"1\", b = \"2\""]
        );
    }

    #[test]
    fn clip_respects_character_boundaries() {
        assert_eq!(clip_to_char_boundary("héllo", 2), "h");
        assert_eq!(clip_to_char_boundary("héllo", 3), "hé");
    }
}
