use rsomics_common::{Result, RsomicsError};

/// Validate CSV quoting the way Go's `encoding/csv` does in strict (non-lazy)
/// mode. The Rust `csv` crate parses quotes leniently — like Go's
/// `LazyQuotes` — and never errors, so csvtk's fail-loud default needs this
/// pre-check. Two conditions are rejected, matching Go:
///
///   * a `"` inside an unquoted field (`ErrBareQuote`), and
///   * a `"` in a quoted field that is neither doubled nor field-terminating,
///     including an unclosed quoted field at EOF (`ErrQuote`).
///
/// Comment and blank lines are skipped exactly as the parser skips them (only
/// at a record boundary), so a `"` inside a comment or a quoted field spanning
/// a `#`-leading continuation line is not misread.
pub fn check_strict(buf: &[u8], delim: u8, comment: Option<u8>) -> Result<()> {
    let n = buf.len();
    let mut i = 0usize;
    let mut line = 1usize;

    'record: while i < n {
        if comment == Some(buf[i]) {
            while i < n && buf[i] != b'\n' {
                i += 1;
            }
            if i < n {
                i += 1;
                line += 1;
            }
            continue 'record;
        }
        if buf[i] == b'\n' {
            i += 1;
            line += 1;
            continue 'record;
        }
        if buf[i] == b'\r' && i + 1 < n && buf[i + 1] == b'\n' {
            i += 2;
            line += 1;
            continue 'record;
        }

        'field: loop {
            if i >= n {
                break 'field;
            }
            if buf[i] != b'"' {
                // Unquoted field: runs to the next delimiter or line end; a
                // quote anywhere in it is bare.
                let start = i;
                while i < n && buf[i] != delim && buf[i] != b'\n' {
                    i += 1;
                }
                let at_delim = i < n && buf[i] == delim;
                let mut end = i;
                if !at_delim && end > start && buf[end - 1] == b'\r' {
                    end -= 1;
                }
                if buf[start..end].contains(&b'"') {
                    return Err(err(line, "bare \" in non-quoted-field"));
                }
                if at_delim {
                    i += 1;
                    continue 'field;
                }
                if i < n {
                    i += 1;
                    line += 1;
                }
                break 'field;
            }

            // Quoted field: consume the opening quote, then walk quote-to-quote.
            i += 1;
            loop {
                let mut j = i;
                while j < n && buf[j] != b'"' {
                    if buf[j] == b'\n' {
                        line += 1;
                    }
                    j += 1;
                }
                if j >= n {
                    return Err(err(line, "extraneous or missing \" in quoted-field"));
                }
                i = j + 1;
                if i < n && buf[i] == b'"' {
                    i += 1; // "" escaped quote
                    continue;
                }
                if i < n && buf[i] == delim {
                    i += 1;
                    continue 'field; // next field
                }
                if i >= n {
                    break 'field; // closing quote at EOF ends the record
                }
                if buf[i] == b'\n' {
                    i += 1;
                    line += 1;
                    break 'field;
                }
                if buf[i] == b'\r' && i + 1 < n && buf[i + 1] == b'\n' {
                    i += 2;
                    line += 1;
                    break 'field;
                }
                if buf[i] == b'\r' && i + 1 == n {
                    i += 1;
                    break 'field;
                }
                return Err(err(line, "extraneous or missing \" in quoted-field"));
            }
        }
    }
    Ok(())
}

fn err(line: usize, what: &str) -> RsomicsError {
    RsomicsError::InvalidInput(format!("parse error on line {line}: {what}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok(s: &str) -> bool {
        check_strict(s.as_bytes(), b',', Some(b'#')).is_ok()
    }

    #[test]
    fn plain_is_ok() {
        assert!(ok("a,b,c\n1,2,3\n"));
    }

    #[test]
    fn quoted_fields_ok() {
        assert!(ok("1,\"x,y\",z\n2,\"he said \"\"hi\"\"\",w\n"));
    }

    #[test]
    fn bare_quote_rejected() {
        assert!(!ok("a,b\n1,x\"y\n2,z\n"));
    }

    #[test]
    fn extraneous_quote_rejected() {
        assert!(!ok("a,b\n1,\"x\"y\n2,z\n"));
    }

    #[test]
    fn unclosed_quote_rejected() {
        assert!(!ok("a,b\n1,\"unclosed\n"));
    }

    #[test]
    fn multiline_quoted_field_ok() {
        assert!(ok("a,b\n1,\"line1\nline2\"\n2,z\n"));
    }

    #[test]
    fn quote_inside_comment_ignored() {
        assert!(ok("# a \" comment\na,b\n1,2\n"));
    }

    #[test]
    fn leading_space_then_quote_is_bare() {
        // The quote is not at the field start, so it is a bare quote.
        assert!(!ok("a,b\n1, \"x\"\n"));
    }

    #[test]
    fn tab_delimiter_quotes_ok() {
        assert!(check_strict(b"a\t\"x\ty\"\n", b'\t', Some(b'#')).is_ok());
    }
}
