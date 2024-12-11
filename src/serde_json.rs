use core::error::Error;
use core::str::Chars;

use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet, LinkedList};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// The internal state of a JSON serialization.
#[non_exhaustive]
pub struct SerJsonState {
    pub out: String,
}

impl SerJsonState {
    pub fn new(out: String) -> Self {
        Self { out }
    }

    pub fn indent(&mut self, _d: usize) {
        //for _ in 0..d {
        //    self.out.push_str("    ");
        //}
    }

    pub fn field(&mut self, d: usize, field: &str) {
        self.indent(d);
        self.out.push('"');
        self.out.push_str(field);
        self.out.push('"');
        self.out.push(':');
    }

    pub fn label(&mut self, label: &str) {
        self.out.push('"');
        self.out.push_str(label);
        self.out.push('"');
    }

    pub fn conl(&mut self) {
        self.out.push(',')
    }

    pub fn st_pre(&mut self) {
        self.out.push('{');
    }

    pub fn st_post(&mut self, d: usize) {
        self.indent(d);
        self.out.push('}');
    }
}

/// A trait for objects that can be serialized to JSON.
pub trait SerJson {
    /// Serialize Self to a JSON string.
    ///
    /// This is a convenient wrapper around `ser_json`.
    fn serialize_json(&self) -> String {
        let mut s = SerJsonState { out: String::new() };
        self.ser_json(0, &mut s);
        s.out
    }

    /// Serialize Self to a JSON string.
    ///
    /// ```rust
    /// # use nanoserde::*;
    /// let mut s = SerJsonState::new(String::new());
    /// 42u32.ser_json(0, &mut s);
    /// assert_eq!(s.out, "42");
    /// ```
    fn ser_json(&self, d: usize, s: &mut SerJsonState);
}

/// A trait for objects that can be deserialized from JSON.
pub trait DeJson: Sized {
    /// Parse Self from the input string.
    ///
    /// This is a convenient wrapper around `de_json`.
    fn deserialize_json(input: &str) -> Result<Self, DeJsonErr> {
        let mut state = DeJsonState::default();
        let mut chars = input.chars();
        state.next(&mut chars);
        state.next_tok(&mut chars)?;
        DeJson::de_json(&mut state, &mut chars)
    }

    /// Parse Self from the input string.
    ///
    /// ```rust
    /// # use nanoserde::*;
    /// let mut state = DeJsonState::default();
    /// let mut chars = "42".chars();
    /// state.next(&mut chars);
    /// state.next_tok(&mut chars).unwrap();
    /// let out = u32::de_json(&mut state, &mut chars).unwrap();
    /// assert_eq!(out, 42);
    /// ```
    fn de_json(state: &mut DeJsonState, input: &mut Chars) -> Result<Self, DeJsonErr>;
}

/// A JSON parsed token.
#[derive(PartialEq, Debug, Default, Clone)]
#[non_exhaustive]
pub enum DeJsonTok {
    Str,
    Char(char),
    U64(u64),
    I64(i64),
    F64(f64),
    Bool(bool),
    BareIdent,
    Null,
    Colon,
    CurlyOpen,
    CurlyClose,
    BlockOpen,
    BlockClose,
    Comma,
    #[default]
    Bof,
    Eof,
}

/// The internal state of a JSON deserialization.
#[derive(Default)]
#[non_exhaustive]
pub struct DeJsonState {
    pub cur: char,
    pub tok: DeJsonTok,
    pub strbuf: String,
    pub numbuf: String,
    pub identbuf: String,
    pub line: usize,
    pub col: usize,
}

#[derive(Clone)]
#[non_exhaustive]
pub enum DeJsonErrReason {
    UnexpectedKey(String),
    UnexpectedToken(DeJsonTok, String),
    MissingKey(String),
    NoSuchEnum(String),
    OutOfRange(String),
    WrongType(String),
    CannotParse(String),
}

/// The error message when failing to deserialize a JSON string.
#[derive(Clone)]
pub struct DeJsonErr {
    pub line: usize,
    pub col: usize,
    pub msg: DeJsonErrReason,
}

impl core::fmt::Debug for DeJsonErrReason {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnexpectedKey(name) => write!(f, "Unexpected key {}", name),
            Self::MissingKey(name) => write!(f, "Key not found {}", name),
            Self::NoSuchEnum(name) => write!(f, "Enum not defined {}", name),
            Self::UnexpectedToken(token, name) => {
                write!(f, "Unexpected token {:?} expected {} ", token, name)
            }
            Self::OutOfRange(value) => write!(f, "Value out of range {} ", value),
            Self::WrongType(found) => write!(f, "Token wrong type {} ", found),
            Self::CannotParse(unparseable) => write!(f, "Cannot parse {} ", unparseable),
        }
    }
}

impl core::fmt::Debug for DeJsonErr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let DeJsonErr {
            line,
            col: column,
            msg: reason,
        } = self;
        write!(
            f,
            "Json Deserialize error: {:?}, line:{} col:{}",
            reason,
            line + 1,
            column + 1
        )
    }
}

impl core::fmt::Display for DeJsonErr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }
}

impl Error for DeJsonErr {}

impl DeJsonState {
    pub fn next(&mut self, i: &mut Chars) {
        if let Some(c) = i.next() {
            self.cur = c;
            if self.cur == '\n' {
                self.line += 1;
                self.col = 0;
            } else {
                self.col += 1;
            }
        } else {
            self.cur = '\0';
        }
    }

    pub fn err_exp(&self, name: &str) -> DeJsonErr {
        DeJsonErr {
            msg: DeJsonErrReason::UnexpectedKey(name.to_string()),
            line: self.line,
            col: self.col,
        }
    }

    pub fn err_nf(&self, name: &str) -> DeJsonErr {
        DeJsonErr {
            msg: DeJsonErrReason::MissingKey(name.to_string()),
            line: self.line,
            col: self.col,
        }
    }

    pub fn err_enum(&self, name: &str) -> DeJsonErr {
        DeJsonErr {
            msg: DeJsonErrReason::NoSuchEnum(name.to_string()),
            line: self.line,
            col: self.col,
        }
    }

    pub fn err_token(&self, what: &str) -> DeJsonErr {
        DeJsonErr {
            msg: DeJsonErrReason::UnexpectedToken(self.tok.clone(), what.to_string()),
            line: self.line,
            col: self.col,
        }
    }

    pub fn err_range(&self, what: &str) -> DeJsonErr {
        DeJsonErr {
            msg: DeJsonErrReason::OutOfRange(what.to_string()),
            line: self.line,
            col: self.col,
        }
    }

    pub fn err_type(&self, what: &str) -> DeJsonErr {
        DeJsonErr {
            msg: DeJsonErrReason::WrongType(what.to_string()),
            line: self.line,
            col: self.col,
        }
    }

    pub fn err_parse(&self, what: &str) -> DeJsonErr {
        DeJsonErr {
            msg: DeJsonErrReason::CannotParse(what.to_string()),
            line: self.line,
            col: self.col,
        }
    }

    pub fn eat_comma_block(&mut self, i: &mut Chars) -> Result<(), DeJsonErr> {
        match self.tok {
            DeJsonTok::Comma => {
                self.next_tok(i)?;
                Ok(())
            }
            DeJsonTok::BlockClose => Ok(()),
            _ => Err(self.err_token(", or ]")),
        }
    }

    pub fn whole_field(&mut self, i: &mut Chars) -> Result<(), DeJsonErr> {
        match self.tok {
            DeJsonTok::F64 { .. }
            | DeJsonTok::I64 { .. }
            | DeJsonTok::Str
            | DeJsonTok::U64 { .. }
            | DeJsonTok::Bool { .. }
            | DeJsonTok::Null => {
                self.next_tok(i)?;
                Ok(())
            }
            DeJsonTok::BlockOpen | DeJsonTok::CurlyOpen => {
                let mut open_brackets = 0;

                loop {
                    if let DeJsonTok::BlockOpen | DeJsonTok::CurlyOpen = self.tok {
                        open_brackets += 1;
                    }

                    if let DeJsonTok::BlockClose | DeJsonTok::CurlyClose = self.tok {
                        open_brackets -= 1;
                    }

                    self.next_tok(i)?;

                    if open_brackets == 0 {
                        break;
                    }
                }
                Ok(())
            }
            _ => unimplemented!("{:?}", self.tok),
        }
    }

    pub fn eat_comma_curly(&mut self, i: &mut Chars) -> Result<(), DeJsonErr> {
        match self.tok {
            DeJsonTok::Comma => {
                self.next_tok(i)?;
                Ok(())
            }
            DeJsonTok::CurlyClose => Ok(()),
            _ => Err(self.err_token(", or }")),
        }
    }

    pub fn colon(&mut self, i: &mut Chars) -> Result<(), DeJsonErr> {
        match self.tok {
            DeJsonTok::Colon => {
                self.next_tok(i)?;
                Ok(())
            }
            _ => Err(self.err_token(":")),
        }
    }

    pub fn string(&mut self, i: &mut Chars) -> Result<(), DeJsonErr> {
        match &mut self.tok {
            DeJsonTok::Str => {
                self.next_tok(i)?;
                Ok(())
            }
            _ => Err(self.err_token("String")),
        }
    }

    pub fn next_colon(&mut self, i: &mut Chars) -> Result<(), DeJsonErr> {
        self.next_tok(i)?;
        self.colon(i)?;
        Ok(())
    }

    pub fn next_str(&mut self) -> Option<()> {
        if let DeJsonTok::Str = &mut self.tok {
            //let mut s = String::new();
            //core::mem::swap(&mut s, name);
            Some(())
        } else {
            None
        }
    }

    pub fn block_open(&mut self, i: &mut Chars) -> Result<(), DeJsonErr> {
        if self.tok == DeJsonTok::BlockOpen {
            self.next_tok(i)?;
            return Ok(());
        }
        Err(self.err_token("["))
    }

    pub fn block_close(&mut self, i: &mut Chars) -> Result<(), DeJsonErr> {
        if self.tok == DeJsonTok::BlockClose {
            self.next_tok(i)?;
            return Ok(());
        }
        Err(self.err_token("]"))
    }

    pub fn curly_open(&mut self, i: &mut Chars) -> Result<(), DeJsonErr> {
        if self.tok == DeJsonTok::CurlyOpen {
            self.next_tok(i)?;
            return Ok(());
        }
        Err(self.err_token("{"))
    }

    pub fn curly_close(&mut self, i: &mut Chars) -> Result<(), DeJsonErr> {
        if self.tok == DeJsonTok::CurlyClose {
            self.next_tok(i)?;
            return Ok(());
        }
        Err(self.err_token("}"))
    }

    pub fn u64_range(&mut self, max: u64) -> Result<u64, DeJsonErr> {
        if let DeJsonTok::U64(value) = self.tok {
            if value > max {
                return Err(self.err_range(&format!("{}>{}", value, max)));
            }
            return Ok(value);
        }
        Err(self.err_token("unsigned integer"))
    }

    pub fn i64_range(&mut self, min: i64, max: i64) -> Result<i64, DeJsonErr> {
        if let DeJsonTok::I64(value) = self.tok {
            if value < min {
                return Err(self.err_range(&format!("{}<{}", value, min)));
            }
            return Ok(value);
        }
        if let DeJsonTok::U64(value) = self.tok {
            if value as i64 > max {
                return Err(self.err_range(&format!("{}>{}", value, max)));
            }
            return Ok(value as i64);
        }
        Err(self.err_token("signed integer"))
    }

    pub fn as_f64(&mut self) -> Result<f64, DeJsonErr> {
        if let DeJsonTok::I64(value) = self.tok {
            return Ok(value as f64);
        }
        if let DeJsonTok::U64(value) = self.tok {
            return Ok(value as f64);
        }
        if let DeJsonTok::F64(value) = self.tok {
            return Ok(value);
        }
        Err(self.err_token("floating point"))
    }

    pub fn as_bool(&mut self) -> Result<bool, DeJsonErr> {
        if let DeJsonTok::Bool(value) = self.tok {
            return Ok(value);
        }
        Err(self.err_token("boolean"))
    }

    pub fn as_string(&mut self) -> Result<String, DeJsonErr> {
        if let DeJsonTok::Str = &mut self.tok {
            let mut val = String::new();
            core::mem::swap(&mut val, &mut self.strbuf);
            return Ok(val);
        }
        Err(self.err_token("string"))
    }

    pub fn next_tok(&mut self, i: &mut Chars) -> Result<(), DeJsonErr> {
        while self.cur == '\n' || self.cur == '\r' || self.cur == '\t' || self.cur == ' ' {
            self.next(i);
        }
        if self.cur == '\0' {
            self.tok = DeJsonTok::Eof;
            return Ok(());
        }
        if self.cur == '/' {
            self.next(i);
            match self.cur {
                '/' => {
                    while self.cur != '\n' && self.cur != '\0' {
                        self.next(i);
                    }
                    return self.next_tok(i);
                }
                '*' => {
                    let mut last = self.cur;
                    loop {
                        self.next(i);
                        if self.cur == '\0' {
                            return Err(self.err_token("MultiLineCommentClose"));
                        }
                        if last == '*' && self.cur == '/' {
                            self.next(i);
                            break;
                        }
                        last = self.cur;
                    }
                    return self.next_tok(i);
                }
                _ => {
                    return Err(self.err_token("CommentOpen"));
                }
            }
        }
        match self.cur {
            ':' => {
                self.next(i);
                self.tok = DeJsonTok::Colon;
                Ok(())
            }
            ',' => {
                self.next(i);
                self.tok = DeJsonTok::Comma;
                Ok(())
            }
            '[' => {
                self.next(i);
                self.tok = DeJsonTok::BlockOpen;
                Ok(())
            }
            ']' => {
                self.next(i);
                self.tok = DeJsonTok::BlockClose;
                Ok(())
            }
            '{' => {
                self.next(i);
                self.tok = DeJsonTok::CurlyOpen;
                Ok(())
            }
            '}' => {
                self.next(i);
                self.tok = DeJsonTok::CurlyClose;
                Ok(())
            }
            '-' | '+' | '0'..='9' => {
                self.numbuf.truncate(0);
                let is_neg = if self.cur == '-' || self.cur == '+' {
                    let sign = self.cur;
                    self.numbuf.push(self.cur);
                    self.next(i);
                    sign == '-'
                } else {
                    false
                };
                while self.cur >= '0' && self.cur <= '9' {
                    self.numbuf.push(self.cur);
                    self.next(i);
                }
                let mut is_float = false;
                if self.cur == '.' {
                    is_float = true;
                    self.numbuf.push(self.cur);
                    self.next(i);
                    while self.cur >= '0' && self.cur <= '9' {
                        self.numbuf.push(self.cur);
                        self.next(i);
                    }
                }
                if self.cur == 'e' || self.cur == 'E' {
                    is_float = true;
                    self.numbuf.push(self.cur);
                    self.next(i);
                    if self.cur == '-' {
                        self.numbuf.push(self.cur);
                        self.next(i);
                    }
                    while self.cur >= '0' && self.cur <= '9' {
                        self.numbuf.push(self.cur);
                        self.next(i);
                    }
                }
                if is_float {
                    if let Ok(num) = self.numbuf.parse() {
                        self.tok = DeJsonTok::F64(num);
                        Ok(())
                    } else {
                        Err(self.err_parse("number"))
                    }
                } else {
                    if is_neg {
                        if let Ok(num) = self.numbuf.parse() {
                            self.tok = DeJsonTok::I64(num);
                            return Ok(());
                        } else {
                            return Err(self.err_parse("number"));
                        }
                    }
                    if let Ok(num) = self.numbuf.parse() {
                        self.tok = DeJsonTok::U64(num);
                        Ok(())
                    } else {
                        Err(self.err_parse("number"))
                    }
                }
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                self.identbuf.truncate(0);
                while self.cur >= 'a' && self.cur <= 'z'
                    || self.cur >= 'A' && self.cur <= 'Z'
                    || self.cur == '_'
                {
                    self.identbuf.push(self.cur);
                    self.next(i);
                }
                if self.identbuf == "true" {
                    self.tok = DeJsonTok::Bool(true);
                    return Ok(());
                }
                if self.identbuf == "false" {
                    self.tok = DeJsonTok::Bool(false);
                    return Ok(());
                }
                if self.identbuf == "null" {
                    self.tok = DeJsonTok::Null;
                    return Ok(());
                }
                self.tok = DeJsonTok::BareIdent;
                Err(self.err_token(&format!(
                    "Got ##{}## needed true, false, null",
                    self.identbuf
                )))
            }
            '"' => {
                self.strbuf.truncate(0);
                self.next(i);
                while self.cur != '"' {
                    if self.cur == '\\' {
                        self.next(i);
                        match self.cur {
                            'n' => self.strbuf.push('\n'),
                            'r' => self.strbuf.push('\r'),
                            't' => self.strbuf.push('\t'),
                            'b' => self.strbuf.push('\x08'),
                            'f' => self.strbuf.push('\x0c'),
                            '0' => self.strbuf.push('\0'),
                            '\0' => {
                                return Err(self.err_parse("string"));
                            }
                            'u' => {
                                if let Some(c) = self.hex_unescape_char(i) {
                                    self.strbuf.push(c);
                                    continue;
                                } else {
                                    return Err(self.err_parse("string"));
                                }
                            }
                            _ => self.strbuf.push(self.cur),
                        }
                        self.next(i);
                    } else {
                        if self.cur == '\0' {
                            return Err(self.err_parse("string"));
                        }
                        self.strbuf.push(self.cur);
                        self.next(i);
                    }
                }
                self.next(i);
                self.tok = DeJsonTok::Str;
                Ok(())
            }
            _ => Err(self.err_token("tokenizer")),
        }
    }

    /// Helper for reading `\uXXXX` escapes out of a string, properly handing
    /// surrogate pairs (by potentially unescaping a second `\uXXXX` sequence if
    /// it would complete a surrogate pair).
    ///
    /// On illegal escapes or unpaired surrogates returns None (and caller
    /// should emit an error).
    fn hex_unescape_char(&mut self, i: &mut Chars) -> Option<char> {
        self.next(i);
        let a = xdigit4(self, i)?;
        if let Some(c) = core::char::from_u32(a as u32) {
            return Some(c);
        }
        // `a` isn't a valid scalar, but if it's leading surrogate, we look for
        // a trailing surrogate in a `\uXXXX` sequence immediately after.
        let a_is_lead = (0xd800..0xdc00).contains(&a);
        if a_is_lead && self.cur == '\\' {
            self.next(i);
            if self.cur == 'u' {
                self.next(i);
                let b = xdigit4(self, i)?;
                let b_is_trail = (0xdc00..0xe000).contains(&b);
                if b_is_trail {
                    // It's a valid pair! We have `[a, b]` where `a` is a leading
                    // surrogate and `b` is a trailing one.
                    let scalar = (((a as u32 - 0xd800) << 10) | (b as u32 - 0xdc00)) + 0x10000;
                    // All valid surrogate pairs decode to unicode scalar values
                    // (e.g. `char`), so this block should always return `Some`, the
                    // debug_assert exists just to ensure our testing is thorough
                    // enough.
                    let ch = core::char::from_u32(scalar);
                    debug_assert!(ch.is_some());
                    return ch;
                }
            }
        }
        return None;

        // Helper to turn next 4 ascii hex digits into a u16
        fn xdigit4(de: &mut DeJsonState, i: &mut Chars) -> Option<u16> {
            // as tempting as it is to try to find a way to use from_str_radix on the
            // next 4 bytes from `i`, we'd still need to do validation to detect cases
            // like `\u+123` and such which makes it less attractive.
            (0..4).try_fold(0u16, |acc, _| {
                let n = match de.cur {
                    '0'..='9' => de.cur as u16 - '0' as u16,
                    'a'..='f' => de.cur as u16 - 'a' as u16 + 10,
                    'A'..='F' => de.cur as u16 - 'A' as u16 + 10,
                    _ => return None,
                };
                de.next(i);
                Some(acc * 16 + n)
            })
        }
    }
}

macro_rules! impl_ser_de_json_unsigned {
    ( $ ty: ident, $ max: expr) => {
        impl SerJson for $ty {
            fn ser_json(&self, _d: usize, s: &mut SerJsonState) {
                s.out.push_str(&self.to_string());
            }
        }

        impl DeJson for $ty {
            fn de_json(s: &mut DeJsonState, i: &mut Chars) -> Result<$ty, DeJsonErr> {
                let val = s.u64_range($max as u64)?;
                s.next_tok(i)?;
                return Ok(val as $ty);
            }
        }
    };
}

macro_rules! impl_ser_de_json_signed {
    ( $ ty: ident, $ min: expr, $ max: expr) => {
        impl SerJson for $ty {
            fn ser_json(&self, _d: usize, s: &mut SerJsonState) {
                s.out.push_str(&self.to_string());
            }
        }

        impl DeJson for $ty {
            fn de_json(s: &mut DeJsonState, i: &mut Chars) -> Result<$ty, DeJsonErr> {
                //s.is_prefix(p, i) ?;
                let val = s.i64_range($min as i64, $max as i64)?;
                s.next_tok(i)?;
                return Ok(val as $ty);
            }
        }
    };
}

macro_rules! impl_ser_de_json_float {
    ( $ ty: ident) => {
        impl SerJson for $ty {
            fn ser_json(&self, _d: usize, s: &mut SerJsonState) {
                s.out.push_str(&format!("{self:?}"));
            }
        }

        impl DeJson for $ty {
            fn de_json(s: &mut DeJsonState, i: &mut Chars) -> Result<$ty, DeJsonErr> {
                //s.is_prefix(p, i) ?;
                let val = s.as_f64()?;
                s.next_tok(i)?;
                return Ok(val as $ty);
            }
        }
    };
}

impl_ser_de_json_unsigned!(usize, u64::MAX);
impl_ser_de_json_unsigned!(u64, u64::MAX);
impl_ser_de_json_unsigned!(u32, u32::MAX);
impl_ser_de_json_unsigned!(u16, u16::MAX);
impl_ser_de_json_unsigned!(u8, u8::MAX);
impl_ser_de_json_signed!(i64, i64::MIN, i64::MAX);
impl_ser_de_json_signed!(i32, i32::MIN, i32::MAX);
impl_ser_de_json_signed!(i16, i16::MIN, i16::MAX);
impl_ser_de_json_signed!(i8, i8::MIN, i8::MAX);
impl_ser_de_json_float!(f64);
impl_ser_de_json_float!(f32);

impl<T> SerJson for Option<T>
where
    T: SerJson,
{
    fn ser_json(&self, d: usize, s: &mut SerJsonState) {
        if let Some(v) = self {
            v.ser_json(d, s);
        } else {
            s.out.push_str("null");
        }
    }
}

impl<T> DeJson for Option<T>
where
    T: DeJson,
{
    fn de_json(s: &mut DeJsonState, i: &mut Chars) -> Result<Self, DeJsonErr> {
        if let DeJsonTok::Null = s.tok {
            s.next_tok(i)?;
            return Ok(None);
        }
        Ok(Some(DeJson::de_json(s, i)?))
    }
}

impl SerJson for () {
    fn ser_json(&self, _d: usize, s: &mut SerJsonState) {
        s.out.push_str("null")
    }
}

impl DeJson for () {
    fn de_json(s: &mut DeJsonState, i: &mut Chars) -> Result<(), DeJsonErr> {
        if let DeJsonTok::Null = s.tok {
            s.next_tok(i)?;
            Ok(())
        } else {
            Err(s.err_token("null"))
        }
    }
}

impl SerJson for bool {
    fn ser_json(&self, _d: usize, s: &mut SerJsonState) {
        if *self {
            s.out.push_str("true")
        } else {
            s.out.push_str("false")
        }
    }
}

impl DeJson for bool {
    fn de_json(s: &mut DeJsonState, i: &mut Chars) -> Result<bool, DeJsonErr> {
        let val = s.as_bool()?;
        s.next_tok(i)?;
        Ok(val)
    }
}

macro_rules! impl_ser_json_string {
    ($ty: ident) => {
        impl SerJson for $ty {
            fn ser_json(&self, _d: usize, s: &mut SerJsonState) {
                s.out.push('"');
                for c in self.chars() {
                    match c {
                        '\x08' => s.out += "\\b",
                        '\x0C' => s.out += "\\f",
                        '\n' => s.out += "\\n",
                        '\r' => s.out += "\\r",
                        '\t' => s.out += "\\t",
                        _ if c.is_ascii_control() => {
                            use core::fmt::Write as _;
                            let _ = write!(s.out, "\\u{:04x}", c as u32);
                        }
                        '\\' => s.out += "\\\\",
                        '"' => s.out += "\\\"",
                        _ => s.out.push(c),
                    }
                }
                s.out.push('"');
            }
        }
    };
}

impl_ser_json_string!(String);
impl_ser_json_string!(str);

impl DeJson for String {
    fn de_json(s: &mut DeJsonState, i: &mut Chars) -> Result<String, DeJsonErr> {
        let val = s.as_string()?;
        s.next_tok(i)?;
        Ok(val)
    }
}

impl<T> SerJson for Vec<T>
where
    T: SerJson,
{
    fn ser_json(&self, d: usize, s: &mut SerJsonState) {
        s.out.push('[');
        if !self.is_empty() {
            let last = self.len() - 1;
            for (index, item) in self.iter().enumerate() {
                s.indent(d + 1);
                item.ser_json(d + 1, s);
                if index != last {
                    s.out.push(',');
                }
            }
        }
        s.out.push(']');
    }
}

impl<T> DeJson for Vec<T>
where
    T: DeJson,
{
    fn de_json(s: &mut DeJsonState, i: &mut Chars) -> Result<Vec<T>, DeJsonErr> {
        let mut out = Vec::new();
        s.block_open(i)?;

        while s.tok != DeJsonTok::BlockClose {
            out.push(DeJson::de_json(s, i)?);
            s.eat_comma_block(i)?;
        }
        s.block_close(i)?;
        Ok(out)
    }
}

#[cfg(feature = "std")]
impl<T> SerJson for std::collections::HashSet<T>
where
    T: SerJson,
{
    fn ser_json(&self, d: usize, s: &mut SerJsonState) {
        s.out.push('[');
        if !self.is_empty() {
            let last = self.len() - 1;
            for (index, item) in self.iter().enumerate() {
                s.indent(d + 1);
                item.ser_json(d + 1, s);
                if index != last {
                    s.out.push(',');
                }
            }
        }
        s.out.push(']');
    }
}

#[cfg(feature = "std")]
impl<T> DeJson for std::collections::HashSet<T>
where
    T: DeJson + core::hash::Hash + Eq,
{
    fn de_json(s: &mut DeJsonState, i: &mut Chars) -> Result<Self, DeJsonErr> {
        let mut out = std::collections::HashSet::new();
        s.block_open(i)?;

        while s.tok != DeJsonTok::BlockClose {
            out.insert(DeJson::de_json(s, i)?);
            s.eat_comma_block(i)?;
        }
        s.block_close(i)?;
        Ok(out)
    }
}

impl<T> SerJson for LinkedList<T>
where
    T: SerJson,
{
    fn ser_json(&self, d: usize, s: &mut SerJsonState) {
        s.out.push('[');
        if !self.is_empty() {
            let last = self.len() - 1;
            for (index, item) in self.iter().enumerate() {
                s.indent(d + 1);
                item.ser_json(d + 1, s);
                if index != last {
                    s.out.push(',');
                }
            }
        }
        s.out.push(']');
    }
}

impl<T> DeJson for LinkedList<T>
where
    T: DeJson,
{
    fn de_json(s: &mut DeJsonState, i: &mut Chars) -> Result<LinkedList<T>, DeJsonErr> {
        let mut out = LinkedList::new();
        s.block_open(i)?;

        while s.tok != DeJsonTok::BlockClose {
            out.push_back(DeJson::de_json(s, i)?);
            s.eat_comma_block(i)?;
        }
        s.block_close(i)?;
        Ok(out)
    }
}

impl<T> SerJson for BTreeSet<T>
where
    T: SerJson,
{
    fn ser_json(&self, d: usize, s: &mut SerJsonState) {
        s.out.push('[');
        if !self.is_empty() {
            let last = self.len() - 1;
            for (index, item) in self.iter().enumerate() {
                s.indent(d + 1);
                item.ser_json(d + 1, s);
                if index != last {
                    s.out.push(',');
                }
            }
        }
        s.out.push(']');
    }
}

impl<T> DeJson for BTreeSet<T>
where
    T: DeJson + Ord,
{
    fn de_json(s: &mut DeJsonState, i: &mut Chars) -> Result<BTreeSet<T>, DeJsonErr> {
        let mut out = BTreeSet::new();
        s.block_open(i)?;

        while s.tok != DeJsonTok::BlockClose {
            out.insert(DeJson::de_json(s, i)?);
            s.eat_comma_block(i)?;
        }
        s.block_close(i)?;
        Ok(out)
    }
}

impl<T> SerJson for [T]
where
    T: SerJson,
{
    fn ser_json(&self, d: usize, s: &mut SerJsonState) {
        s.out.push('[');
        let last = self.len() - 1;
        for (index, item) in self.iter().enumerate() {
            item.ser_json(d + 1, s);
            if index != last {
                s.out.push(',');
            }
        }
        s.out.push(']');
    }
}

impl<T, const N: usize> SerJson for [T; N]
where
    T: SerJson,
{
    #[inline(always)]
    fn ser_json(&self, d: usize, s: &mut SerJsonState) {
        self.as_slice().ser_json(d, s)
    }
}

impl<T, const N: usize> DeJson for [T; N]
where
    T: DeJson,
{
    fn de_json(o: &mut DeJsonState, d: &mut Chars) -> Result<Self, DeJsonErr> {
        use core::mem::MaybeUninit;

        // waiting for uninit_array(or for array::try_from_fn) stabilization
        // https://github.com/rust-lang/rust/issues/96097
        // https://github.com/rust-lang/rust/issues/89379
        let mut to: [MaybeUninit<T>; N] =
            unsafe { MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init() };
        o.block_open(d)?;

        for index in 0..N {
            to[index] = match DeJson::de_json(o, d).and_then(|ret| {
                o.eat_comma_block(d)?;
                Ok(ret)
            }) {
                Ok(v) => MaybeUninit::new(v),
                Err(e) => {
                    // drop all the MaybeUninit values which we've already
                    // successfully deserialized so we don't leak memory.
                    // See https://github.com/not-fl3/nanoserde/issues/79
                    for (_, to_drop) in (0..index).zip(to) {
                        unsafe { to_drop.assume_init() };
                    }
                    return Err(e);
                }
            }
        }

        // waiting for array_assume_init or core::array::map optimizations
        // https://github.com/rust-lang/rust/issues/61956
        // initializing before block close so that drop will run automatically if err encountered there
        let initialized =
            unsafe { (*(&to as *const _ as *const MaybeUninit<_>)).assume_init_read() };
        o.block_close(d)?;

        Ok(initialized)
    }
}

fn de_json_comma_block<T>(s: &mut DeJsonState, i: &mut Chars) -> Result<T, DeJsonErr>
where
    T: DeJson,
{
    let t = DeJson::de_json(s, i);
    s.eat_comma_block(i)?;
    t
}

impl<A, B> SerJson for (A, B)
where
    A: SerJson,
    B: SerJson,
{
    fn ser_json(&self, d: usize, s: &mut SerJsonState) {
        s.out.push('[');
        self.0.ser_json(d, s);
        s.out.push(',');
        self.1.ser_json(d, s);
        s.out.push(']');
    }
}

impl<A, B> DeJson for (A, B)
where
    A: DeJson,
    B: DeJson,
{
    fn de_json(s: &mut DeJsonState, i: &mut Chars) -> Result<(A, B), DeJsonErr> {
        s.block_open(i)?;
        let r = (de_json_comma_block(s, i)?, de_json_comma_block(s, i)?);
        s.block_close(i)?;
        Ok(r)
    }
}

impl<A, B, C> SerJson for (A, B, C)
where
    A: SerJson,
    B: SerJson,
    C: SerJson,
{
    fn ser_json(&self, d: usize, s: &mut SerJsonState) {
        s.out.push('[');
        self.0.ser_json(d, s);
        s.out.push(',');
        self.1.ser_json(d, s);
        s.out.push(',');
        self.2.ser_json(d, s);
        s.out.push(']');
    }
}

impl<A, B, C> DeJson for (A, B, C)
where
    A: DeJson,
    B: DeJson,
    C: DeJson,
{
    fn de_json(s: &mut DeJsonState, i: &mut Chars) -> Result<(A, B, C), DeJsonErr> {
        s.block_open(i)?;
        let r = (
            de_json_comma_block(s, i)?,
            de_json_comma_block(s, i)?,
            de_json_comma_block(s, i)?,
        );
        s.block_close(i)?;
        Ok(r)
    }
}

impl<A, B, C, D> SerJson for (A, B, C, D)
where
    A: SerJson,
    B: SerJson,
    C: SerJson,
    D: SerJson,
{
    fn ser_json(&self, d: usize, s: &mut SerJsonState) {
        s.out.push('[');
        self.0.ser_json(d, s);
        s.out.push(',');
        self.1.ser_json(d, s);
        s.out.push(',');
        self.2.ser_json(d, s);
        s.out.push(',');
        self.3.ser_json(d, s);
        s.out.push(']');
    }
}

impl<A, B, C, D> DeJson for (A, B, C, D)
where
    A: DeJson,
    B: DeJson,
    C: DeJson,
    D: DeJson,
{
    fn de_json(s: &mut DeJsonState, i: &mut Chars) -> Result<(A, B, C, D), DeJsonErr> {
        s.block_open(i)?;
        let r = (
            de_json_comma_block(s, i)?,
            de_json_comma_block(s, i)?,
            de_json_comma_block(s, i)?,
            de_json_comma_block(s, i)?,
        );
        s.block_close(i)?;
        Ok(r)
    }
}

#[cfg(feature = "std")]
impl<K, V> SerJson for std::collections::HashMap<K, V>
where
    K: SerJson,
    V: SerJson,
{
    fn ser_json(&self, d: usize, s: &mut SerJsonState) {
        s.out.push('{');
        let len = self.len();
        for (index, (k, v)) in self.iter().enumerate() {
            s.indent(d + 1);
            k.ser_json(d + 1, s);
            s.out.push(':');
            v.ser_json(d + 1, s);
            if (index + 1) < len {
                s.conl();
            }
        }
        s.indent(d);
        s.out.push('}');
    }
}

#[cfg(feature = "std")]
impl<K, V> DeJson for std::collections::HashMap<K, V>
where
    K: DeJson + Eq + core::hash::Hash,
    V: DeJson,
{
    fn de_json(s: &mut DeJsonState, i: &mut Chars) -> Result<Self, DeJsonErr> {
        let mut h = std::collections::HashMap::new();
        s.curly_open(i)?;
        while s.tok != DeJsonTok::CurlyClose {
            let k = DeJson::de_json(s, i)?;
            s.colon(i)?;
            let v = DeJson::de_json(s, i)?;
            s.eat_comma_curly(i)?;
            h.insert(k, v);
        }
        s.curly_close(i)?;
        Ok(h)
    }
}

impl<K, V> SerJson for BTreeMap<K, V>
where
    K: SerJson,
    V: SerJson,
{
    fn ser_json(&self, d: usize, s: &mut SerJsonState) {
        s.out.push('{');
        let len = self.len();
        for (index, (k, v)) in self.iter().enumerate() {
            s.indent(d + 1);
            k.ser_json(d + 1, s);
            s.out.push(':');
            v.ser_json(d + 1, s);
            if (index + 1) < len {
                s.conl();
            }
        }
        s.indent(d);
        s.out.push('}');
    }
}

impl<K, V> DeJson for BTreeMap<K, V>
where
    K: DeJson + Eq + Ord,
    V: DeJson,
{
    fn de_json(s: &mut DeJsonState, i: &mut Chars) -> Result<Self, DeJsonErr> {
        let mut h = BTreeMap::new();
        s.curly_open(i)?;
        while s.tok != DeJsonTok::CurlyClose {
            let k = DeJson::de_json(s, i)?;
            s.colon(i)?;
            let v = DeJson::de_json(s, i)?;
            s.eat_comma_curly(i)?;
            h.insert(k, v);
        }
        s.curly_close(i)?;
        Ok(h)
    }
}

impl<T> SerJson for Box<T>
where
    T: SerJson,
{
    fn ser_json(&self, d: usize, s: &mut SerJsonState) {
        (**self).ser_json(d, s)
    }
}

impl<T> DeJson for Box<T>
where
    T: DeJson,
{
    fn de_json(s: &mut DeJsonState, i: &mut Chars) -> Result<Box<T>, DeJsonErr> {
        Ok(Box::new(DeJson::de_json(s, i)?))
    }
}
