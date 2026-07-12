//! Go `encoding/csv`-exact I/O primitives shared by the `rsomics-tsv-*` tools.
//!
//! csvtk reads and writes through Go's `encoding/csv`, whose quoting and
//! strict-parse behaviour differs from the Rust `csv` crate's defaults. Two
//! pieces capture that gap: [`CsvWriter`] serialises a record byte-for-byte the
//! way Go's writer does, and [`check_strict`] reproduces Go's non-lazy parser so
//! a malformed quote fails loud instead of being silently recovered. Pair them
//! with a lenient `csv::Reader` for the actual field splitting, after running
//! [`normalize_crlf`] over the raw bytes to match Go's CRLF→LF normalisation.

mod newline;
mod quotes;
mod reader;
mod writer;

pub use newline::normalize_crlf;
pub use quotes::check_strict;
pub use reader::go_reader_builder;
pub use writer::CsvWriter;
