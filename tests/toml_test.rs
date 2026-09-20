use std::collections::{hash_map, BTreeMap, HashMap};

use nanoserde::{Toml, TomlParser};
use toml_test_harness::{DecodedScalar, DecodedValue, Decoder, DecoderHarness, Error};

#[test]
fn decoder_harness() {
    /// The supported TOML version
    const TOML_VERSION: &str = "1.0.0";

    let mut harness = DecoderHarness::new(NanoSerde);
    harness.version(TOML_VERSION);
    harness
        .ignore(
            EMPTY_TABLE
                .into_iter()
                .chain(SINGLE_VALUE_TABLE_ARR)
                .chain(CANNOT_PARSE)
                .chain(IMPROPER_ESCAPE)
                .chain(EXTRA_WS_IN_MULTILINE)
                .chain(NUMERIC_KEY)
                .chain(AMBIGUOUS_NUMBER)
                .chain(BAD_NUMBERS)
                .chain(SHOULD_BE_INVALID)
                .chain(ARRAY_NESTING)
                .chain(DOTTED_KEY)
                .chain(KEYWORD_TABLE)
                .chain(BAD_DATE)
                .copied(),
        )
        .unwrap();
    harness.test();
}

/// Empty tables are entirely omitted
const EMPTY_TABLE: &[&str] = &[
    "valid/table/empty.toml",
    "valid/table/no-eol.toml",
    "valid/table/sub-empty.toml",
    "valid/table/without-super.toml",
    "valid/table/whitespace.toml",
    "valid/spec-1.0.0/table-[0456].toml",
];

/// Both regular tables and arrays of tables are represented through the same value, so we assume
/// an array with a length of one is always a table even though sometimes it's an array
const SINGLE_VALUE_TABLE_ARR: &[&str] = &[
    "valid/table/array-{one,empty,implicit,table-array,within-dotted}.toml",
    "valid/table/array-implicit-and-explicit-after.toml",
    "valid/spec-1.0.0/array-of-tables-1.toml",
];

/// > Toml error: Cannot parse toml tokenizer , line:# col:#
const CANNOT_PARSE: &[&str] = &[
    "valid/key/dotted-0[12].toml",
    "valid/key/dotted-empty.toml",
    "valid/key/empty-0[123].toml",
    "valid/table/with-literal-string.toml",
    "valid/table/with-single-quotes.toml",
    "valid/table/without-single-quotes.toml",
    "valid/table/names.toml",
    "valid/table/names-with-values.toml",
    "valid/table/array-empty-name.toml",
    "valid/string/start-mb.toml",
    "valid/string/raw.toml",
    "valid/string/raw-multiline.toml",
    "valid/string/raw-empty.toml",
    "valid/string/quoted-unicode.toml",
    "valid/string/nl.toml",
    "valid/string/multiline-quotes.toml",
    "valid/spec-1.0.0/table-[23].toml",
    "valid/spec-1.0.0/string-[567].toml",
    "valid/spec-1.0.0/offset-date-time-1.toml",
    "valid/spec-1.0.0/local-time-0.toml",
    "valid/spec-1.0.0/local-date-time-0.toml",
    "valid/spec-1.0.0/keys-[13].toml",
    "valid/spec-1.0.0/inline-table-[02].toml",
    "valid/spec-1.0.0/float-0.toml",
    "valid/spec-1.0.0/array-of-tables-2.toml",
    "valid/spec-1.0.0/array-0.toml",
    "valid/multibyte.toml",
    "valid/key/start.toml",
    "valid/key/quoted-unicode.toml",
    "valid/key/numeric-08.toml",
    "valid/key/like-date.toml",
    "valid/key/escapes.toml",
    "valid/inline-table/spaces.toml",
    "valid/inline-table/nest.toml",
    "valid/inline-table/multiline.toml",
    "valid/table/empty-name.toml",
    "valid/string/empty.toml",
    "valid/inline-table/key-dotted-0[1234567].toml",
    "valid/inline-table/inline-table.toml",
    "valid/inline-table/end-in-bool.toml",
    "valid/inline-table/empty.toml",
    "valid/inline-table/bool.toml",
    "valid/inline-table/array-0[123].toml",
    "valid/datetime/milliseconds.toml",
    "valid/datetime/local.toml",
    "valid/datetime/local-time.toml",
    "valid/datetime/invalid-date-in-string.toml",
    "valid/datetime/edge.toml",
    "valid/datetime/datetime.toml",
    "valid/datetime/tricky.toml",
    "valid/comment/after-literal-no-ws.toml",
    "valid/array/table-array-string-backslash.toml",
    "valid/array/strings.toml",
    "valid/array/nested-inline-table.toml",
    "valid/array/mixed-string-table.toml",
    "valid/array/bool.toml",
    "valid/array/array.toml",
    "valid/string/multibyte.toml",
    "valid/string/escape-tricky.toml",
    "valid/comment/tricky.toml",
];

/// Some escaped characters aren't parsed as escaped characters
const IMPROPER_ESCAPE: &[&str] = &[
    "valid/key/space.toml",
    "valid/string/{unicode,multibyte}-escape.toml",
    "valid/string/escapes.toml",
    "valid/spec-1.0.0/string-[024].toml",
];

/// Multiline strings accidentally include extra whitespace
const EXTRA_WS_IN_MULTILINE: &[&str] = &[
    "valid/string/multiline.toml",
    "valid/string/multiline-{empty,escaped-crlf}.toml",
    "valid/string/multibyte.toml",
    "valid/string/ends-in-whitespace-escape.toml",
    "valid/spec-1.0.0/string-[13].toml",
];

/// Tables with keyword names are rejected
const KEYWORD_TABLE: &[&str] = &[
    "valid/table/keyword.toml",
    "valid/table/keyword-with-values.toml",
];

/// Several date and/or time related issues. Primarily with not recognizing timezones
const BAD_DATE: &[&str] = &[
    "valid/datetime/leap-year.toml",
    "valid/datetime/timezone.toml",
    "valid/spec-1.0.0/offset-date-time-0.toml",
    "valid/example.toml",
];

/// Keys that look like numbers are treated as numbers even though they're strings (leading zeros
/// are stripped :/)
const NUMERIC_KEY: &[&str] = &["valid/key/numeric-0[347].toml", "valid/key/alphanum.toml"];

/// All numbers are decoded to floats despite toml having a separate integer type. To hack around
/// this we assume all whole numbers are integers. These tests are where thats incorrect
const AMBIGUOUS_NUMBER: &[&str] = &["valid/float/max-int.toml"];

const BAD_NUMBERS: &[&str] = &[
    // Floats with an `e` exponent are rejected
    "valid/float/{exponent,underscore,zero}.toml",
    // Numbers using an `0x` of `0b` prefix are rejected
    "valid/integer/{literals,zero}.toml",
    "valid/spec-1.0.0/integer-2.toml",
];

/// There are a myriad of issues when dealing with nesting anything in arrays
const ARRAY_NESTING: &[&str] = &[
    "valid/table/array-nest.toml",
    "valid/array/array-subtables.toml",
    "valid/array/open-parent-table.toml",
];

const DOTTED_KEY: &[&str] = &[
    // A quoted key can include a dot, but given that nanoserde flattens keys without fully
    // perserving quoting. It becomes impossible to distinguish between a key containing a dot and
    // the key being split by a dot
    "valid/key/quoted-dots.toml",
    // This one was too complex for me to feel comfortable hacking into the dotted key splitting in
    // the harness
    "valid/key/special-chars.toml",
    // Needs to ignore whitespace between dot and key
    "valid/spec-1.0.0/keys-4.toml",
];

/// I'm too lazy to classify these further for now. These should be invalid, but they're accepted
const SHOULD_BE_INVALID: &[&str] = &[
    "invalid/array/double-comma-0[12].toml",
    "invalid/array/extend-defined-aot.toml",
    "invalid/array/missing-separator-0[12].toml",
    "invalid/array/no-close-0[1238].toml",
    "invalid/array/no-comma-0[123].toml",
    "invalid/array/only-comma-0[12].toml",
    "invalid/array/tables-0[12].toml",
    "invalid/control/bare-{cr,null}.toml",
    "invalid/control/comment-{cr,del,ff,lf,null,us}.toml",
    "invalid/control/multi-{cr,del,lf,us}.toml",
    "invalid/control/only-null.toml",
    "invalid/control/string-{bs,cr,del,lf,us}.toml",
    "invalid/datetime/{hour,mday,minute,month}-over.toml",
    "invalid/datetime/{mday,month}-under.toml",
    "invalid/datetime/no-{date-time,year-month}-sep.toml",
    "invalid/datetime/second-over.toml",
    "invalid/encoding/utf16-{comment,key}.toml",
    "invalid/float/leading-{dot,zero}-{neg,plus}.toml",
    "invalid/float/leading-zero.toml",
    "invalid/float/trailing-dot-{01,02,min,plus}.toml",
    "invalid/float/trailing-{dot,us}.toml",
    "invalid/float/us-{after,before}-dot.toml",
    "invalid/integer/double-{sign-nex,us}.toml",
    "invalid/integer/leading-zero-0[123].toml",
    "invalid/integer/leading-zero-sign-0[123].toml",
    "invalid/integer/trailing-us.toml",
    "invalid/key/after-{array,table,value}.toml",
    "invalid/key/dotted-redefine-table-0[12].toml",
    "invalid/key/duplicate-keys-0[123568].toml",
    "invalid/key/multiline-key-0[13].toml",
    "invalid/key/newline-0[1246].toml",
    "invalid/key/no-eol-0[14].toml",
    "invalid/key/special-character.toml",
    "invalid/local-date/day-1digit.toml",
    "invalid/local-date/feb-{29,30}.toml",
    "invalid/local-date/mday-{over,under}.toml",
    "invalid/local-date/month-{over,under}.toml",
    "invalid/local-date/no-leads-with-milli.toml",
    "invalid/local-date/no-leads.toml",
    "invalid/local-date/trailing-t.toml",
    "invalid/local-date/y10k.toml",
    "invalid/local-date/year-3digits.toml",
    "invalid/local-datetime/feb-{29,30}.toml",
    "invalid/local-datetime/{hour,mday,minute,month,second}-over.toml",
    "invalid/local-datetime/{mday,month}-under.toml",
    "invalid/local-datetime/no-{leads,secs,t}.toml",
    "invalid/spec-1.0.0/table-9-[01].toml",
    "invalid/string/bad-byte-escape.toml",
    "invalid/string/bad-escape-0[1245].toml",
    "invalid/string/bad-hex-esc-0[12345].toml",
    "invalid/string/bad-multiline.toml",
    "invalid/string/bad-slash-escape.toml",
    "invalid/string/bad-uni-esc-0[1234567].toml",
    "invalid/string/bad-uni-esc-ml-0[1234567].toml",
    "invalid/string/basic-byte-escapes.toml",
    "invalid/string/basic-multiline-out-of-range-unicode-escape-0[12].toml",
    "invalid/string/basic-multiline-unknown-escape.toml",
    "invalid/string/basic-out-of-range-unicode-escape-0[12].toml",
    "invalid/string/basic-unknown-escape.toml",
    "invalid/string/multiline-bad-escape-0[123].toml",
    "invalid/string/multiline-escape-space-0[12].toml",
    "invalid/string/no-close-09.toml",
    "invalid/table/append-with-dotted-keys-0[123456].toml",
    "invalid/table/array-implicit.toml",
    "invalid/table/duplicate-key-0[12345679].toml",
    "invalid/table/duplicate-key-10.toml",
    "invalid/table/empty-implicit-table.toml",
    "invalid/table/llbrace.toml",
    "invalid/table/newline-0[1234].toml",
    "invalid/table/overwrite-array-in-parent.toml",
    "invalid/table/overwrite-bool-with-array.toml",
    "invalid/table/overwrite-with-deep-table.toml",
    "invalid/table/redefine-0[123].toml",
    "invalid/table/rrbrace.toml",
    "invalid/table/super-twice.toml",
    "invalid/table/trailing-dot.toml",
];

#[derive(Copy, Clone)]
struct NanoSerde;

impl Decoder for NanoSerde {
    fn name(&self) -> &str {
        "nanoserde::TomlParser"
    }

    fn decode(&self, data: &[u8]) -> Result<DecodedValue, Error> {
        let s = str::from_utf8(data).map_err(Error::new)?;
        let table = TomlParser::parse(s).map_err(Error::new)?;
        value_to_decoded(Toml::Array(vec![table]))
    }
}

/// `nanoserde::Toml`'s structure differs from what `toml_test_harness::DecodedValue` expects. This
/// adapter serves to try and patch over those issues as best it can. Namely:
///
/// - `nanoserde` represents nested tables through dotting the keys themselves while `DecodedValue`
///   nests the table in the value's structure itself
/// - `nanoserde` represents all numbers through a single float (like JSON), whereas TOML has
///   distinct integer/float types. As a hack; floats that are whole numbers are assumed to be ints
/// - `nanoserde` returns both single tables along with arrays of tables through `Toml::Array`. This
///   means that a regular table looks the same as an array containing one table. As a hack; if an
///   array contains only one table then we assume it's just a table, not an array
fn value_to_decoded(toml: Toml) -> Result<DecodedValue, Error> {
    let value = match toml {
        Toml::Str(s) => DecodedValue::Scalar(DecodedScalar::String(s)),
        Toml::Bool(b) => DecodedValue::Scalar(DecodedScalar::Bool(b.to_string())),
        Toml::Num(n) => {
            // HACK: guess which floats are really ints
            let n_int = n as i64;
            let scalar = if n_int as f64 == n {
                DecodedScalar::Integer(n_int.to_string())
            } else {
                DecodedScalar::Float(n.to_string().to_lowercase())
            };
            DecodedValue::Scalar(scalar)
        }
        Toml::Date(d) => {
            // determine the type based on them having distinct lengths
            let scalar = match d.len() {
                10 => DecodedScalar::DateLocal(d),
                25 => DecodedScalar::Datetime(d),
                len => panic!("fill in type matching for len: {len}, date: {d}"),
            };
            DecodedValue::Scalar(scalar)
        }
        Toml::Array(mut tables) => {
            // HACK: if there is only one table then we assume it's a regular table
            if tables.len() == 1 {
                let table = table_to_decoded(tables.pop().expect("checked len"))?;
                DecodedValue::Table(table)
            } else {
                let mut arr = Vec::new();
                for table in tables {
                    arr.push(DecodedValue::Table(table_to_decoded(table)?));
                }
                DecodedValue::Array(arr)
            }
        }
        Toml::SimpleArray(arr) => DecodedValue::Array(
            arr.into_iter()
                .map(value_to_decoded)
                .collect::<Result<_, _>>()?,
        ),
    };
    Ok(value)
}

fn table_to_decoded(table: BTreeMap<String, Toml>) -> Result<HashMap<String, DecodedValue>, Error> {
    let mut out = HashMap::new();
    for (keys, value) in table {
        if keys.contains(['"', '\'']) && keys.contains('.') {
            return Err(Error::new(format!(
                "Refusing to parse dotted and quoted keys: {keys}'"
            )));
        }
        let value = value_to_decoded(value)?;
        let rev_keys = keys.split('.').rev().map(ToOwned::to_owned).collect();
        insert_nested_key_value(&mut out, rev_keys, value)?;
    }
    Ok(out)
}

fn insert_nested_key_value(
    table: &mut HashMap<String, DecodedValue>,
    mut rev_keys: Vec<String>,
    value: DecodedValue,
) -> Result<(), Error> {
    let Some(key) = rev_keys.pop() else {
        panic!("only call with non-empty list of keys!");
    };
    let entry = table.entry(key);
    if rev_keys.is_empty() {
        // last level. insert the value
        match entry {
            hash_map::Entry::Vacant(v) => {
                v.insert(value);
                Ok(())
            }
            hash_map::Entry::Occupied(_) => {
                Err(Error::new("Attempted to insert value into existing value"))
            }
        }
    } else {
        // nested. recurse deeper
        let inner = entry.or_insert_with(|| DecodedValue::Table(HashMap::new()));
        match inner {
            DecodedValue::Table(inner) => insert_nested_key_value(inner, rev_keys, value),
            _other => Err(Error::new("Attempted to insert table into existing value")),
        }
    }
}
