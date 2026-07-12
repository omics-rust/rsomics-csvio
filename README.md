# rsomics-csvio

Go `encoding/csv`-exact CSV/TSV I/O primitives shared by the `rsomics-tsv-*`
family (ports of [csvtk](https://github.com/shenwei356/csvtk)).

csvtk reads and writes through Go's `encoding/csv`. Its quoting and strict-parse
rules differ from the Rust `csv` crate's defaults, so byte-for-byte parity needs
two dedicated pieces:

- **`CsvWriter`** — serialises each record the way Go's `csv.Writer` does: a
  field is quoted iff it is non-empty and contains the delimiter, a `"`, `\r`,
  or `\n`, or begins with a Unicode-whitespace rune, or is the literal `\.`;
  inner `"` doubles; records end with `\n`.
- **`check_strict`** — reproduces Go's non-lazy parser (`ErrBareQuote` /
  `ErrQuote`) so a bare or unbalanced quote errors loudly instead of being
  leniently recovered. Run it over the raw bytes before handing them to a
  lenient `csv::Reader`.

This is an internal foundation crate; the public surface is intentionally small.

## License

MIT OR Apache-2.0
