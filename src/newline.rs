/// Convert every `\r\n` to `\n`, matching Go `encoding/csv`, whose reader
/// normalises all CRLF sequences to LF before returning records — *including
/// inside quoted multi-line fields*. The Rust `csv` crate preserves `\r\n`
/// verbatim inside a quoted field, so callers must run this over the raw input
/// first to stay byte-exact with csvtk. A lone `\r` mid-field (an old-Mac line
/// ending, or a stray carriage return inside a field) is left untouched, exactly
/// as Go leaves it — except that Go's reader also drops a single `\r` at the
/// very end of the input when the final line has no trailing `\n` (its
/// "drop trailing \r before EOF" backwards-compatibility rule), so one trailing
/// carriage return is popped here too.
pub fn normalize_crlf(buf: Vec<u8>) -> Vec<u8> {
    if !buf.contains(&b'\r') {
        return buf;
    }
    let mut out = Vec::with_capacity(buf.len());
    let mut i = 0;
    while i < buf.len() {
        if buf[i] == b'\r' && buf.get(i + 1) == Some(&b'\n') {
            out.push(b'\n');
            i += 2;
        } else {
            out.push(buf[i]);
            i += 1;
        }
    }
    if out.last() == Some(&b'\r') {
        out.pop();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn norm(s: &[u8]) -> Vec<u8> {
        normalize_crlf(s.to_vec())
    }

    #[test]
    fn record_terminators_normalized() {
        assert_eq!(norm(b"a,b\r\n1,2\r\n"), b"a,b\n1,2\n");
    }

    #[test]
    fn crlf_inside_quoted_field_normalized() {
        assert_eq!(norm(b"\"line1\r\nline2\",2\r\n"), b"\"line1\nline2\",2\n");
    }

    #[test]
    fn lone_cr_untouched() {
        assert_eq!(norm(b"a\rb\n"), b"a\rb\n");
    }

    #[test]
    fn trailing_lone_cr_at_eof_stripped() {
        // Go drops a single trailing \r when the last line has no \n.
        assert_eq!(norm(b"ab\r"), b"ab");
    }

    #[test]
    fn double_trailing_cr_strips_one() {
        assert_eq!(norm(b"ab\r\r"), b"ab\r");
    }

    #[test]
    fn mid_field_cr_before_trailing_cr() {
        // Only the very last \r is dropped; a mid-field \r survives.
        assert_eq!(norm(b"a\rb\r"), b"a\rb");
    }

    #[test]
    fn no_cr_returns_input_unchanged() {
        assert_eq!(norm(b"a,b\n1,2\n"), b"a,b\n1,2\n");
    }

    #[test]
    fn consecutive_crlf() {
        assert_eq!(norm(b"a\r\n\r\nb"), b"a\n\nb");
    }
}
