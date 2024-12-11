use core::error::Error;
use core::str::Chars;

use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet, LinkedList};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// The internal state of a RON serialization.
pub struct SerRonState {
    pub out: String,
}

impl SerRonState {
    pub fn indent(&mut self, d: usize) {
        for _ in 0..d {
            self.out.push_str("    ");
        }
    }

    pub fn field(&mut self, d: usize, field: &str) {
        self.indent(d);
        self.out.push_str(field);
        self.out.push(':');
    }

    pub fn conl(&mut self) {
        self.out.push_str(",\n")
    }

    pub fn st_pre(&mut self) {
        self.out.push_str("(\n");
    }

    pub fn st_post(&mut self, d: usize) {
        self.indent(d);
        self.out.push(')');
    }
}

/// A trait for objects that can be serialized to the RON file format.
///
/// [Specification](https://github.com/ron-rs/ron).
pub trait SerRon {
    /// Serialize Self to a RON string.
    ///
    /// This is a convenient wrapper around `ser_ron`.
    fn serialize_ron(&self) -> String {
        let mut s = SerRonState { out: String::new() };
        self.ser_ron(0, &mut s);
        s.out
    }

    /// Serialize Self to a RON string.
    ///
    /// ```rust
    /// # use nanoserde::*;
    /// let mut s = SerRonState { out: String::new() };
    /// 42u32.ser_ron(0, &mut s);
    /// assert_eq!(s.out, "42");
    /// ```
    fn ser_ron(&self, indent_level: usize, state: &mut SerRonState);
}

/// A trait for objects that can be deserialized from the RON file format.
///
/// [Specification](https://github.com/ron-rs/ron).
pub trait DeRon: Sized {
    /// Parse Self from a RON string.
    ///
    /// This is a convenient wrapper around `de_ron`.
    fn deserialize_ron(input: &str) -> Result<Self, DeRonErr> {
        let mut state = DeRonState::default();
        let mut chars = input.chars();
        state.next(&mut chars);
        state.next_tok(&mut chars)?;
        DeRon::de_ron(&mut state, &mut chars)
    }

    /// Parse Self from a RON string.
    ///
    /// ```rust
    /// # use nanoserde::*;
    /// let mut state = DeRonState::default();
    /// let mut chars = "42".chars();
    /// state.next(&mut chars);
    /// state.next_tok(&mut chars).unwrap();
    /// let out = u32::de_ron(&mut state, &mut chars).unwrap();
    /// assert_eq!(out, 42);
    /// ```
    fn de_ron(state: &mut DeRonState, input: &mut Chars) -> Result<Self, DeRonErr>;
}

/// A RON parsed token.
#[derive(PartialEq, Debug, Default, Clone)]
pub enum DeRonTok {
    Ident,
    Str,
    U64(u64),
    I64(i64),
    F64(f64),
    Bool(bool),
    Char(char),
    Colon,
    CurlyOpen,
    CurlyClose,
    ParenOpen,
    ParenClose,
    BlockOpen,
    BlockClose,
    Comma,
    #[default]
    Bof,
    Eof,
}

/// The internal state of a RON deserialization.
#[derive(Default)]
#[non_exhaustive]
pub struct DeRonState {
    pub cur: char,
    pub tok: DeRonTok,
    pub strbuf: String,
    pub numbuf: String,
    pub identbuf: String,
    pub line: usize,
    pub col: usize,
}

#[derive(Clone)]
#[non_exhaustive]
pub enum DeRonErrReason {
    UnexpectedKey(String),
    UnexpectedToken(DeRonTok, String),
    MissingKey(String),
    NoSuchEnum(String),
    OutOfRange(String),
    WrongType(String),
    CannotParse(String),
}

/// The error message when failing to deserialize a Ron string.
#[derive(Clone)]
pub struct DeRonErr {
    pub line: usize,
    pub col: usize,
    pub msg: DeRonErrReason,
}

impl core::fmt::Debug for DeRonErrReason {
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

impl core::fmt::Debug for DeRonErr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let DeRonErr {
            line,
            col: column,
            msg: reason,
        } = self;
        write!(
            f,
            "Ron Deserialize error: {:?}, line:{} col:{}",
            reason,
            line + 1,
            column + 1
        )
    }
}

impl core::fmt::Display for DeRonErr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }
}

impl Error for DeRonErr {}

impl DeRonState {
    pub fn next(&mut self, i: &mut Chars) {
        if let Some(c) = i.next() {
            self.cur = c;
            if self.cur == '\n' {
                self.line += 1;
                self.col = 0;
            } else {
                self.col = 0;
            }
        } else {
            self.cur = '\0';
        }
    }

    pub fn err_exp(&self, name: &str) -> DeRonErr {
        DeRonErr {
            msg: DeRonErrReason::UnexpectedKey(name.to_string()),
            line: self.line,
            col: self.col,
        }
    }

    pub fn err_nf(&self, name: &str) -> DeRonErr {
        DeRonErr {
            msg: DeRonErrReason::MissingKey(name.to_string()),
            line: self.line,
            col: self.col,
        }
    }

    pub fn err_enum(&self, name: &str) -> DeRonErr {
        DeRonErr {
            msg: DeRonErrReason::NoSuchEnum(name.to_string()),
            line: self.line,
            col: self.col,
        }
    }

    pub fn err_token(&self, what: &str) -> DeRonErr {
        DeRonErr {
            msg: DeRonErrReason::UnexpectedToken(self.tok.clone(), what.to_string()),
            line: self.line,
            col: self.col,
        }
    }

    pub fn err_range(&self, what: &str) -> DeRonErr {
        DeRonErr {
            msg: DeRonErrReason::OutOfRange(what.to_string()),
            line: self.line,
            col: self.col,
        }
    }

    pub fn err_type(&self, what: &str) -> DeRonErr {
        DeRonErr {
            msg: DeRonErrReason::WrongType(what.to_string()),
            line: self.line,
            col: self.col,
        }
    }

    pub fn err_parse(&self, what: &str) -> DeRonErr {
        DeRonErr {
            msg: DeRonErrReason::CannotParse(what.to_string()),
            line: self.line,
            col: self.col,
        }
    }

    pub fn eat_comma_paren(&mut self, i: &mut Chars) -> Result<(), DeRonErr> {
        match self.tok {
            DeRonTok::Comma => {
                self.next_tok(i)?;
                Ok(())
            }
            DeRonTok::ParenClose => Ok(()),
            _ => Err(self.err_token(", or )")),
        }
    }

    pub fn eat_comma_block(&mut self, i: &mut Chars) -> Result<(), DeRonErr> {
        match self.tok {
            DeRonTok::Comma => {
                self.next_tok(i)?;
                Ok(())
            }
            DeRonTok::BlockClose => Ok(()),
            _ => Err(self.err_token(", or ]")),
        }
    }

    pub fn eat_comma_curly(&mut self, i: &mut Chars) -> Result<(), DeRonErr> {
        match self.tok {
            DeRonTok::Comma => {
                self.next_tok(i)?;
                Ok(())
            }
            DeRonTok::CurlyClose => Ok(()),
            _ => Err(self.err_token(", or }")),
        }
    }

    pub fn colon(&mut self, i: &mut Chars) -> Result<(), DeRonErr> {
        match self.tok {
            DeRonTok::Colon => {
                self.next_tok(i)?;
                Ok(())
            }
            _ => Err(self.err_token(":")),
        }
    }

    pub fn ident(&mut self, i: &mut Chars) -> Result<(), DeRonErr> {
        match &mut self.tok {
            DeRonTok::Ident => {
                self.next_tok(i)?;
                Ok(())
            }
            _ => Err(self.err_token("Identifier")),
        }
    }

    pub fn next_colon(&mut self, i: &mut Chars) -> Result<(), DeRonErr> {
        self.next_tok(i)?;
        self.colon(i)?;
        Ok(())
    }

    pub fn next_ident(&mut self) -> Option<()> {
        if let DeRonTok::Ident = &mut self.tok {
            Some(())
        } else {
            None
        }
    }

    pub fn paren_open(&mut self, i: &mut Chars) -> Result<(), DeRonErr> {
        if self.tok == DeRonTok::ParenOpen {
            self.next_tok(i)?;
            return Ok(());
        }
        Err(self.err_token("("))
    }

    pub fn paren_close(&mut self, i: &mut Chars) -> Result<(), DeRonErr> {
        if self.tok == DeRonTok::ParenClose {
            self.next_tok(i)?;
            return Ok(());
        }
        Err(self.err_token(")"))
    }

    pub fn block_open(&mut self, i: &mut Chars) -> Result<(), DeRonErr> {
        if self.tok == DeRonTok::BlockOpen {
            self.next_tok(i)?;
            return Ok(());
        }
        Err(self.err_token("["))
    }

    pub fn block_close(&mut self, i: &mut Chars) -> Result<(), DeRonErr> {
        if self.tok == DeRonTok::BlockClose {
            self.next_tok(i)?;
            return Ok(());
        }
        Err(self.err_token("]"))
    }

    pub fn curly_open(&mut self, i: &mut Chars) -> Result<(), DeRonErr> {
        if self.tok == DeRonTok::CurlyOpen {
            self.next_tok(i)?;
            return Ok(());
        }
        Err(self.err_token("{"))
    }

    pub fn curly_close(&mut self, i: &mut Chars) -> Result<(), DeRonErr> {
        if self.tok == DeRonTok::CurlyClose {
            self.next_tok(i)?;
            return Ok(());
        }
        Err(self.err_token("}"))
    }

    pub fn u64_range(&mut self, max: u64) -> Result<u64, DeRonErr> {
        if let DeRonTok::U64(value) = self.tok {
            if value > max {
                return Err(self.err_range(&format!("{}>{}", value, max)));
            }
            return Ok(value);
        }
        Err(self.err_token("unsigned integer"))
    }

    pub fn i64_range(&mut self, min: i64, max: i64) -> Result<i64, DeRonErr> {
        if let DeRonTok::I64(value) = self.tok {
            if value < min {
                return Err(self.err_range(&format!("{}<{}", value, min)));
            }
            return Ok(value);
        }
        if let DeRonTok::U64(value) = self.tok {
            if value as i64 > max {
                return Err(self.err_range(&format!("{}>{}", value, max)));
            }
            return Ok(value as i64);
        }
        Err(self.err_token("signed integer"))
    }

    pub fn as_f64(&mut self) -> Result<f64, DeRonErr> {
        if let DeRonTok::I64(value) = self.tok {
            return Ok(value as f64);
        }
        if let DeRonTok::U64(value) = self.tok {
            return Ok(value as f64);
        }
        if let DeRonTok::F64(value) = self.tok {
            return Ok(value);
        }
        Err(self.err_token("floating point"))
    }

    pub fn as_bool(&mut self) -> Result<bool, DeRonErr> {
        if let DeRonTok::Bool(value) = self.tok {
            return Ok(value);
        }
        if let DeRonTok::U64(value) = self.tok {
            return Ok(value != 0);
        }
        Err(self.err_token("boolean"))
    }

    pub fn as_string(&mut self) -> Result<String, DeRonErr> {
        if let DeRonTok::Str = &mut self.tok {
            let mut val = String::new();
            core::mem::swap(&mut val, &mut self.strbuf);
            return Ok(val);
        }
        Err(self.err_token("string"))
    }

    pub fn next_tok(&mut self, i: &mut Chars) -> Result<(), DeRonErr> {
        loop {
            while self.cur == '\n' || self.cur == '\r' || self.cur == '\t' || self.cur == ' ' {
                self.next(i);
            }
            match self.cur {
                '\0' => {
                    self.tok = DeRonTok::Eof;
                    return Ok(());
                }
                ':' => {
                    self.next(i);
                    self.tok = DeRonTok::Colon;
                    return Ok(());
                }
                ',' => {
                    self.next(i);
                    self.tok = DeRonTok::Comma;
                    return Ok(());
                }
                '[' => {
                    self.next(i);
                    self.tok = DeRonTok::BlockOpen;
                    return Ok(());
                }
                ']' => {
                    self.next(i);
                    self.tok = DeRonTok::BlockClose;
                    return Ok(());
                }
                '(' => {
                    self.next(i);
                    self.tok = DeRonTok::ParenOpen;
                    return Ok(());
                }
                ')' => {
                    self.next(i);
                    self.tok = DeRonTok::ParenClose;
                    return Ok(());
                }
                '{' => {
                    self.next(i);
                    self.tok = DeRonTok::CurlyOpen;
                    return Ok(());
                }
                '}' => {
                    self.next(i);
                    self.tok = DeRonTok::CurlyClose;
                    return Ok(());
                }
                '/' => {
                    self.next(i);
                    if self.cur == '/' {
                        // single line comment
                        while self.cur != '\0' {
                            if self.cur == '\n' {
                                self.next(i);
                                break;
                            }
                            self.next(i);
                        }
                    } else if self.cur == '*' {
                        // multline comment
                        let mut last_star = false;
                        while self.cur != '\0' {
                            if self.cur == '/' && last_star {
                                self.next(i);
                                break;
                            }
                            last_star = self.cur == '*';
                            self.next(i);
                        }
                    } else {
                        return Err(self.err_parse("comment"));
                    }
                }
                '.' | '-' | '+' | '0'..='9' => {
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
                            self.tok = DeRonTok::F64(num);
                            return Ok(());
                        } else {
                            return Err(self.err_parse("number"));
                        }
                    } else {
                        if is_neg {
                            if let Ok(num) = self.numbuf.parse() {
                                self.tok = DeRonTok::I64(num);
                                return Ok(());
                            } else {
                                return Err(self.err_parse("number"));
                            }
                        }
                        if let Ok(num) = self.numbuf.parse() {
                            self.tok = DeRonTok::U64(num);
                            return Ok(());
                        } else {
                            return Err(self.err_parse("number"));
                        }
                    }
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    self.identbuf.truncate(0);
                    while self.cur >= 'a' && self.cur <= 'z'
                        || self.cur >= 'A' && self.cur <= 'Z'
                        || self.cur == '_'
                        || self.cur >= '0' && self.cur <= '9'
                    {
                        self.identbuf.push(self.cur);
                        self.next(i);
                    }
                    if self.identbuf == "true" {
                        self.tok = DeRonTok::Bool(true);
                        return Ok(());
                    }
                    if self.identbuf == "false" {
                        self.tok = DeRonTok::Bool(false);
                        return Ok(());
                    }
                    self.tok = DeRonTok::Ident;
                    return Ok(());
                }
                '\'' => {
                    self.next(i);
                    if self.cur == '\\' {
                        self.next(i);
                    }
                    let chr = self.cur;
                    self.next(i);
                    if self.cur != '\'' {
                        return Err(self.err_token("char"));
                    }
                    self.next(i);
                    self.tok = DeRonTok::Char(chr);
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
                    self.tok = DeRonTok::Str;
                    return Ok(());
                }
                _ => {
                    return Err(self.err_token("tokenizer"));
                }
            }
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
        fn xdigit4(de: &mut DeRonState, i: &mut Chars) -> Option<u16> {
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

macro_rules! impl_ser_de_ron_unsigned {
    ( $ ty: ident, $ max: expr) => {
        impl SerRon for $ty {
            fn ser_ron(&self, _d: usize, s: &mut SerRonState) {
                s.out.push_str(&self.to_string());
            }
        }

        impl DeRon for $ty {
            fn de_ron(s: &mut DeRonState, i: &mut Chars) -> Result<$ty, DeRonErr> {
                //s.is_prefix(p, i) ?;
                let val = s.u64_range($max as u64)?;
                s.next_tok(i)?;
                return Ok(val as $ty);
            }
        }
    };
}

macro_rules! impl_ser_de_ron_signed {
    ( $ ty: ident, $ min: expr, $ max: expr) => {
        impl SerRon for $ty {
            fn ser_ron(&self, _d: usize, s: &mut SerRonState) {
                s.out.push_str(&self.to_string());
            }
        }

        impl DeRon for $ty {
            fn de_ron(s: &mut DeRonState, i: &mut Chars) -> Result<$ty, DeRonErr> {
                //s.is_prefix(p, i) ?;
                let val = s.i64_range($min as i64, $max as i64)?;
                s.next_tok(i)?;
                return Ok(val as $ty);
            }
        }
    };
}

macro_rules! impl_ser_de_ron_float {
    ( $ ty: ident) => {
        impl SerRon for $ty {
            fn ser_ron(&self, _d: usize, s: &mut SerRonState) {
                s.out.push_str(&format!("{self:?}"));
            }
        }

        impl DeRon for $ty {
            fn de_ron(s: &mut DeRonState, i: &mut Chars) -> Result<$ty, DeRonErr> {
                //s.is_prefix(p, i) ?;
                let val = s.as_f64()?;
                s.next_tok(i)?;
                return Ok(val as $ty);
            }
        }
    };
}

impl_ser_de_ron_unsigned!(usize, u64::MAX);
impl_ser_de_ron_unsigned!(u64, u64::MAX);
impl_ser_de_ron_unsigned!(u32, u32::MAX);
impl_ser_de_ron_unsigned!(u16, u16::MAX);
impl_ser_de_ron_unsigned!(u8, u8::MAX);
impl_ser_de_ron_signed!(i64, i64::MIN, i64::MAX);
impl_ser_de_ron_signed!(i32, i32::MIN, i32::MAX);
impl_ser_de_ron_signed!(i16, i16::MIN, i16::MAX);
impl_ser_de_ron_signed!(i8, i8::MIN, i8::MAX);
impl_ser_de_ron_float!(f64);
impl_ser_de_ron_float!(f32);

impl<T> SerRon for Option<T>
where
    T: SerRon,
{
    fn ser_ron(&self, d: usize, s: &mut SerRonState) {
        if let Some(v) = self {
            v.ser_ron(d, s);
        } else {
            s.out.push_str("None");
        }
    }
}

impl<T> DeRon for Option<T>
where
    T: DeRon,
{
    fn de_ron(s: &mut DeRonState, i: &mut Chars) -> Result<Self, DeRonErr> {
        if let DeRonTok::Ident = &s.tok {
            if s.identbuf == "None" {
                s.next_tok(i)?;
                return Ok(None);
            }
        }
        Ok(Some(DeRon::de_ron(s, i)?))
    }
}

impl SerRon for bool {
    fn ser_ron(&self, _d: usize, s: &mut SerRonState) {
        if *self {
            s.out.push_str("true")
        } else {
            s.out.push_str("false")
        }
    }
}

impl DeRon for bool {
    fn de_ron(s: &mut DeRonState, i: &mut Chars) -> Result<bool, DeRonErr> {
        let val = s.as_bool()?;
        s.next_tok(i)?;
        Ok(val)
    }
}

impl SerRon for String {
    fn ser_ron(&self, _d: usize, s: &mut SerRonState) {
        s.out.push('"');
        for c in self.chars() {
            match c {
                '\n' => {
                    s.out.push('\\');
                    s.out.push('n');
                }
                '\r' => {
                    s.out.push('\\');
                    s.out.push('r');
                }
                '\t' => {
                    s.out.push('\\');
                    s.out.push('t');
                }
                '\0' => {
                    s.out.push('\\');
                    s.out.push('0');
                }
                '\\' => {
                    s.out.push('\\');
                    s.out.push('\\');
                }
                '"' => {
                    s.out.push('\\');
                    s.out.push('"');
                }
                _ => s.out.push(c),
            }
        }
        s.out.push('"');
    }
}

impl DeRon for String {
    fn de_ron(s: &mut DeRonState, i: &mut Chars) -> Result<String, DeRonErr> {
        let val = s.as_string()?;
        s.next_tok(i)?;
        Ok(val)
    }
}

impl<T> SerRon for Vec<T>
where
    T: SerRon,
{
    fn ser_ron(&self, d: usize, s: &mut SerRonState) {
        s.out.push_str("[\n");
        for item in self {
            s.indent(d + 1);
            item.ser_ron(d + 1, s);
            s.conl();
        }
        s.indent(d);
        s.out.push(']');
    }
}

impl<T> DeRon for Vec<T>
where
    T: DeRon,
{
    fn de_ron(s: &mut DeRonState, i: &mut Chars) -> Result<Vec<T>, DeRonErr> {
        let mut out = Vec::new();
        s.block_open(i)?;

        while s.tok != DeRonTok::BlockClose {
            out.push(DeRon::de_ron(s, i)?);
            s.eat_comma_block(i)?;
        }
        s.block_close(i)?;
        Ok(out)
    }
}

#[cfg(feature = "std")]
impl<T> SerRon for std::collections::HashSet<T>
where
    T: SerRon,
{
    fn ser_ron(&self, d: usize, s: &mut SerRonState) {
        s.out.push('[');
        if !self.is_empty() {
            let last = self.len() - 1;
            for (index, item) in self.iter().enumerate() {
                s.indent(d + 1);
                item.ser_ron(d + 1, s);
                if index != last {
                    s.out.push(',');
                }
            }
        }
        s.out.push(']');
    }
}

#[cfg(feature = "std")]
impl<T> DeRon for std::collections::HashSet<T>
where
    T: DeRon + core::hash::Hash + Eq,
{
    fn de_ron(s: &mut DeRonState, i: &mut Chars) -> Result<Self, DeRonErr> {
        let mut out = std::collections::HashSet::new();
        s.block_open(i)?;

        while s.tok != DeRonTok::BlockClose {
            out.insert(DeRon::de_ron(s, i)?);
            s.eat_comma_block(i)?;
        }
        s.block_close(i)?;
        Ok(out)
    }
}

impl<T> SerRon for LinkedList<T>
where
    T: SerRon,
{
    fn ser_ron(&self, d: usize, s: &mut SerRonState) {
        s.out.push('[');
        if !self.is_empty() {
            let last = self.len() - 1;
            for (index, item) in self.iter().enumerate() {
                s.indent(d + 1);
                item.ser_ron(d + 1, s);
                if index != last {
                    s.out.push(',');
                }
            }
        }
        s.out.push(']');
    }
}

impl<T> DeRon for LinkedList<T>
where
    T: DeRon,
{
    fn de_ron(s: &mut DeRonState, i: &mut Chars) -> Result<LinkedList<T>, DeRonErr> {
        let mut out = LinkedList::new();
        s.block_open(i)?;

        while s.tok != DeRonTok::BlockClose {
            out.push_back(DeRon::de_ron(s, i)?);
            s.eat_comma_block(i)?;
        }
        s.block_close(i)?;
        Ok(out)
    }
}

impl<T> SerRon for BTreeSet<T>
where
    T: SerRon,
{
    fn ser_ron(&self, d: usize, s: &mut SerRonState) {
        s.out.push('[');
        if !self.is_empty() {
            let last = self.len() - 1;
            for (index, item) in self.iter().enumerate() {
                s.indent(d + 1);
                item.ser_ron(d + 1, s);
                if index != last {
                    s.out.push(',');
                }
            }
        }
        s.out.push(']');
    }
}

impl<T> DeRon for BTreeSet<T>
where
    T: DeRon + Ord,
{
    fn de_ron(s: &mut DeRonState, i: &mut Chars) -> Result<BTreeSet<T>, DeRonErr> {
        let mut out = BTreeSet::new();
        s.block_open(i)?;

        while s.tok != DeRonTok::BlockClose {
            out.insert(DeRon::de_ron(s, i)?);
            s.eat_comma_block(i)?;
        }
        s.block_close(i)?;
        Ok(out)
    }
}

impl<T> SerRon for [T]
where
    T: SerRon,
{
    fn ser_ron(&self, d: usize, s: &mut SerRonState) {
        s.out.push('(');
        let last = self.len() - 1;
        for (index, item) in self.iter().enumerate() {
            item.ser_ron(d + 1, s);
            if index != last {
                s.out.push_str(", ");
            }
        }
        s.out.push(')');
    }
}

impl<T, const N: usize> SerRon for [T; N]
where
    T: SerRon,
{
    #[inline(always)]
    fn ser_ron(&self, d: usize, s: &mut SerRonState) {
        self.as_slice().ser_ron(d, s)
    }
}

impl<T, const N: usize> DeRon for [T; N]
where
    T: DeRon,
{
    fn de_ron(o: &mut DeRonState, d: &mut Chars) -> Result<Self, DeRonErr> {
        use core::mem::MaybeUninit;

        // waiting for uninit_array(or for array::try_from_fn) stabilization
        // https://github.com/rust-lang/rust/issues/96097
        // https://github.com/rust-lang/rust/issues/89379
        let mut to: [MaybeUninit<T>; N] =
            unsafe { MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init() };
        o.paren_open(d)?;

        for index in 0..N {
            to[index] = match DeRon::de_ron(o, d).and_then(|ret| {
                o.eat_comma_paren(d)?;
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
        o.paren_close(d)?;

        Ok(initialized)
    }
}

fn de_ron_comma_paren<T>(s: &mut DeRonState, i: &mut Chars) -> Result<T, DeRonErr>
where
    T: DeRon,
{
    let t = DeRon::de_ron(s, i);
    s.eat_comma_paren(i)?;
    t
}

impl SerRon for () {
    fn ser_ron(&self, _d: usize, s: &mut SerRonState) {
        s.out.push_str("()");
    }
}

impl DeRon for () {
    fn de_ron(s: &mut DeRonState, i: &mut Chars) -> Result<(), DeRonErr> {
        s.paren_open(i)?;
        s.paren_close(i)?;
        Ok(())
    }
}

impl<A, B> SerRon for (A, B)
where
    A: SerRon,
    B: SerRon,
{
    fn ser_ron(&self, d: usize, s: &mut SerRonState) {
        s.out.push('(');
        self.0.ser_ron(d, s);
        s.out.push_str(", ");
        self.1.ser_ron(d, s);
        s.out.push(')');
    }
}

impl<A, B> DeRon for (A, B)
where
    A: DeRon,
    B: DeRon,
{
    fn de_ron(s: &mut DeRonState, i: &mut Chars) -> Result<(A, B), DeRonErr> {
        s.paren_open(i)?;
        let r = (de_ron_comma_paren(s, i)?, de_ron_comma_paren(s, i)?);
        s.paren_close(i)?;
        Ok(r)
    }
}

impl<A, B, C> SerRon for (A, B, C)
where
    A: SerRon,
    B: SerRon,
    C: SerRon,
{
    fn ser_ron(&self, d: usize, s: &mut SerRonState) {
        s.out.push('(');
        self.0.ser_ron(d, s);
        s.out.push_str(", ");
        self.1.ser_ron(d, s);
        s.out.push_str(", ");
        self.2.ser_ron(d, s);
        s.out.push(')');
    }
}

impl<A, B, C> DeRon for (A, B, C)
where
    A: DeRon,
    B: DeRon,
    C: DeRon,
{
    fn de_ron(s: &mut DeRonState, i: &mut Chars) -> Result<(A, B, C), DeRonErr> {
        s.paren_open(i)?;
        let r = (
            de_ron_comma_paren(s, i)?,
            de_ron_comma_paren(s, i)?,
            de_ron_comma_paren(s, i)?,
        );
        s.paren_close(i)?;
        Ok(r)
    }
}

impl<A, B, C, D> SerRon for (A, B, C, D)
where
    A: SerRon,
    B: SerRon,
    C: SerRon,
    D: SerRon,
{
    fn ser_ron(&self, d: usize, s: &mut SerRonState) {
        s.out.push('(');
        self.0.ser_ron(d, s);
        s.out.push_str(", ");
        self.1.ser_ron(d, s);
        s.out.push_str(", ");
        self.2.ser_ron(d, s);
        s.out.push_str(", ");
        self.3.ser_ron(d, s);
        s.out.push(')');
    }
}

impl<A, B, C, D> DeRon for (A, B, C, D)
where
    A: DeRon,
    B: DeRon,
    C: DeRon,
    D: DeRon,
{
    fn de_ron(s: &mut DeRonState, i: &mut Chars) -> Result<(A, B, C, D), DeRonErr> {
        s.paren_open(i)?;
        let r = (
            de_ron_comma_paren(s, i)?,
            de_ron_comma_paren(s, i)?,
            de_ron_comma_paren(s, i)?,
            de_ron_comma_paren(s, i)?,
        );
        s.paren_close(i)?;
        Ok(r)
    }
}

#[cfg(feature = "std")]
impl<K, V> SerRon for std::collections::HashMap<K, V>
where
    K: SerRon,
    V: SerRon,
{
    fn ser_ron(&self, d: usize, s: &mut SerRonState) {
        s.out.push_str("{\n");
        for (k, v) in self {
            s.indent(d + 1);
            k.ser_ron(d + 1, s);
            s.out.push(':');
            v.ser_ron(d + 1, s);
            s.conl();
        }
        s.indent(d);
        s.out.push('}');
    }
}

#[cfg(feature = "std")]
impl<K, V> DeRon for std::collections::HashMap<K, V>
where
    K: DeRon + Eq + core::hash::Hash,
    V: DeRon,
{
    fn de_ron(s: &mut DeRonState, i: &mut Chars) -> Result<Self, DeRonErr> {
        let mut h = std::collections::HashMap::new();
        s.curly_open(i)?;
        while s.tok != DeRonTok::CurlyClose {
            let k = DeRon::de_ron(s, i)?;
            s.colon(i)?;
            let v = DeRon::de_ron(s, i)?;
            s.eat_comma_curly(i)?;
            h.insert(k, v);
        }
        s.curly_close(i)?;
        Ok(h)
    }
}

impl<K, V> SerRon for BTreeMap<K, V>
where
    K: SerRon,
    V: SerRon,
{
    fn ser_ron(&self, d: usize, s: &mut SerRonState) {
        s.out.push_str("{\n");
        for (k, v) in self {
            s.indent(d + 1);
            k.ser_ron(d + 1, s);
            s.out.push(':');
            v.ser_ron(d + 1, s);
            s.conl();
        }
        s.indent(d);
        s.out.push('}');
    }
}

impl<K, V> DeRon for BTreeMap<K, V>
where
    K: DeRon + Eq + Ord,
    V: DeRon,
{
    fn de_ron(s: &mut DeRonState, i: &mut Chars) -> Result<Self, DeRonErr> {
        let mut h = BTreeMap::new();
        s.curly_open(i)?;
        while s.tok != DeRonTok::CurlyClose {
            let k = DeRon::de_ron(s, i)?;
            s.colon(i)?;
            let v = DeRon::de_ron(s, i)?;
            s.eat_comma_curly(i)?;
            h.insert(k, v);
        }
        s.curly_close(i)?;
        Ok(h)
    }
}

impl<T> SerRon for Box<T>
where
    T: SerRon,
{
    fn ser_ron(&self, d: usize, s: &mut SerRonState) {
        (**self).ser_ron(d, s)
    }
}

impl<T> DeRon for Box<T>
where
    T: DeRon,
{
    fn de_ron(s: &mut DeRonState, i: &mut Chars) -> Result<Box<T>, DeRonErr> {
        Ok(Box::new(DeRon::de_ron(s, i)?))
    }
}
