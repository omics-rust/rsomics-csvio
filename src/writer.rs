use std::io::Write;

use rsomics_common::{Result, RsomicsError};

/// A CSV/TSV writer that reproduces Go's `encoding/csv` byte-for-byte — csvtk
/// writes through that package, and the Rust `csv` crate's default quoting
/// diverges from it. Each record is joined with `delim` and terminated by
/// `\n` (Go's default, `UseCRLF == false`).
pub struct CsvWriter<W: Write> {
    inner: W,
    delim: u8,
}

impl<W: Write> CsvWriter<W> {
    pub fn new(inner: W, delim: u8) -> Self {
        Self { inner, delim }
    }

    pub fn write_record<S: AsRef<str>>(&mut self, fields: &[S]) -> Result<()> {
        let mut line = Vec::with_capacity(64);
        for (i, f) in fields.iter().enumerate() {
            if i > 0 {
                line.push(self.delim);
            }
            encode_field(f.as_ref(), self.delim, &mut line);
        }
        line.push(b'\n');
        self.inner.write_all(&line).map_err(RsomicsError::Io)
    }

    pub fn flush(&mut self) -> Result<()> {
        self.inner.flush().map_err(RsomicsError::Io)
    }
}

/// Go's `Writer.fieldNeedsQuotes`: never quote empty; always quote a bare
/// `\.` (COPY-mode sentinel); quote when the field carries the delimiter, a
/// quote, `\r` or `\n`; finally quote when the first rune is Unicode
/// whitespace. `char::is_whitespace` is the Unicode White_Space property,
/// the same set Go's `unicode.IsSpace` tests.
fn needs_quotes(field: &str, delim: u8) -> bool {
    if field.is_empty() {
        return false;
    }
    if field == r"\." {
        return true;
    }
    let d = delim as char;
    if field
        .chars()
        .any(|c| c == '\n' || c == '\r' || c == '"' || c == d)
    {
        return true;
    }
    field.chars().next().is_some_and(char::is_whitespace)
}

fn encode_field(field: &str, delim: u8, out: &mut Vec<u8>) {
    if !needs_quotes(field, delim) {
        out.extend_from_slice(field.as_bytes());
        return;
    }
    // Wrap in quotes; inner `"` doubles. `\r`/`\n` pass through unchanged
    // (Go only rewrites them when UseCRLF is set, which csvtk never does).
    out.push(b'"');
    for b in field.bytes() {
        if b == b'"' {
            out.extend_from_slice(b"\"\"");
        } else {
            out.push(b);
        }
    }
    out.push(b'"');
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enc(fields: &[&str], delim: u8) -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let mut w = CsvWriter::new(&mut buf, delim);
            w.write_record(fields).unwrap();
        }
        buf
    }

    #[test]
    fn plain_fields_unquoted() {
        assert_eq!(enc(&["1", "abc", "z"], b','), b"1,abc,z\n");
    }

    #[test]
    fn comma_field_quoted() {
        assert_eq!(enc(&["1", "x,y", "z"], b','), b"1,\"x,y\",z\n");
    }

    #[test]
    fn inner_quotes_doubled() {
        assert_eq!(
            enc(&["2", "he said \"hi\"", "w"], b','),
            b"2,\"he said \"\"hi\"\"\",w\n"
        );
    }

    #[test]
    fn empty_never_quoted_but_trailing_space_is() {
        // `3,, ` -> empty middle stays bare, the single-space field is quoted.
        assert_eq!(enc(&["3", "", " "], b','), b"3,,\" \"\n");
    }

    #[test]
    fn leading_space_quoted_trailing_space_not() {
        assert_eq!(
            enc(&["4", " lead", "trail "], b','),
            b"4,\" lead\",trail \n"
        );
    }

    #[test]
    fn backslash_dot_quoted() {
        assert_eq!(enc(&[r"\."], b','), b"\"\\.\"\n");
    }

    #[test]
    fn tab_only_special_when_delimiter() {
        // In CSV mode a tab inside a field is not special...
        assert_eq!(enc(&["a\tb"], b','), b"a\tb\n");
        // ...but in TSV mode it forces quoting.
        assert_eq!(enc(&["a\tb"], b'\t'), b"\"a\tb\"\n");
    }

    #[test]
    fn leading_tab_quoted_in_csv() {
        // A leading tab is Unicode whitespace, so it quotes even in CSV mode.
        assert_eq!(enc(&["\tx"], b','), b"\"\tx\"\n");
    }

    #[test]
    fn newline_field_quoted_not_rewritten() {
        assert_eq!(enc(&["a\nb"], b','), b"\"a\nb\"\n");
    }
}
