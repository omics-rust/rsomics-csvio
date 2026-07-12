use csv::{ReaderBuilder, Terminator};

/// A `csv::ReaderBuilder` configured to split records the way Go `encoding/csv`
/// does, which is what csvtk reads through. The decisive difference from the
/// Rust `csv` crate's defaults is the terminator: Go ends a record only on
/// `\n`, so a bare `\r` — one not part of a `\r\n`, which [`normalize_crlf`]
/// has already collapsed — is ordinary field content. The csv crate's default
/// `Terminator::CRLF` would instead treat that lone `\r` as a record break and
/// silently split the row. Pinning the terminator to `\n` keeps the two in
/// agreement.
///
/// `has_headers(false)` because the tools drive header handling themselves.
/// Finish with `.from_reader(&data[..])`.
///
/// [`normalize_crlf`]: crate::normalize_crlf
pub fn go_reader_builder(delimiter: u8, comment: Option<u8>, flexible: bool) -> ReaderBuilder {
    let mut b = ReaderBuilder::new();
    b.delimiter(delimiter)
        .comment(comment)
        .flexible(flexible)
        .has_headers(false)
        .terminator(Terminator::Any(b'\n'));
    b
}

#[cfg(test)]
mod tests {
    use super::*;

    fn records(data: &[u8]) -> Vec<Vec<String>> {
        go_reader_builder(b',', Some(b'#'), false)
            .from_reader(data)
            .records()
            .map(|r| r.unwrap().iter().map(str::to_owned).collect())
            .collect()
    }

    #[test]
    fn lone_cr_is_field_content() {
        assert_eq!(
            records(b"v\na\rb\n"),
            vec![vec!["v".to_string()], vec!["a\rb".to_string()]]
        );
    }

    #[test]
    fn newline_ends_record() {
        assert_eq!(records(b"a,b\n1,2\n").len(), 2);
    }

    #[test]
    fn comment_line_skipped() {
        assert_eq!(records(b"# note\na\n").len(), 1);
    }
}
