use core::str::Chars;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::{vec, vec::Vec};

#[cfg(feature = "no_std")]
use hashbrown::HashMap;

#[cfg(not(feature = "no_std"))]
use std::collections::HashMap;

/// Pattern matching any valid bare key character as u32.
/// ABNF line: https://github.com/toml-lang/toml/blob/2431aa308a7bc97eeb50673748606e23a6e0f201/toml.abnf#L55
macro_rules! bare_key_chars {
    () => {
        0x41..=0x5A
        | 0x61..=0x7A
        | 0x30..=0x39
        | 0x2D
        | 0x5F
        | 0xB2
        | 0xB3
        | 0xB9
        | 0xBC..=0xBE
        | 0xC0..=0xD6
        | 0xD8..=0xF6
        | 0xF8..=0x37D
        | 0x37F..=0x1FFF
        | 0x200C..=0x200D
        | 0x203F..=0x2040
        | 0x2070..=0x218F
        | 0x2460..=0x24FF
        | 0x2C00..=0x2FEF
        | 0x3001..=0xD7FF
        | 0xF900..=0xFDCF
        | 0xFDF0..=0xFFFD
        | 0x10000..=0xEFFFF
    }
}

/// A parser for TOML string values.
///
/// ```rust
/// # use nanoserde::*;
/// let toml = "[Section]\nvalue=1";
/// let parsed = TomlParser::parse(toml).unwrap();
/// assert_eq!(parsed["Section.value"], Toml::Num(1.));
/// ```
#[derive(Default)]
pub struct TomlParser {
    cur: char,
    line: usize,
    col: usize,
}

/// A TOML parsed token.
#[derive(PartialEq, Debug)]
pub enum TomlTok {
    Ident(String),
    Str(String),
    U64(u64),
    I64(i64),
    F64(f64),
    Bool(bool),
    Nan(bool),
    Inf(bool),
    Date(String),
    Equals,
    BlockOpen,
    BlockClose,
    Comma,
    Bof,
    Eof,
}

/// A TOML value.
#[derive(Debug, PartialEq)]
pub enum Toml {
    Str(String),
    Bool(bool),
    Num(f64),
    Date(String),
    Array(Vec<HashMap<String, Toml>>),
    SimpleArray(Vec<Toml>),
}

impl core::ops::Index<usize> for Toml {
    type Output = HashMap<String, Toml>;

    fn index(&self, index: usize) -> &Self::Output {
        match self {
            Toml::Array(array) => &array[index],
            _ => panic!(),
        }
    }
}

impl Toml {
    /// Get the TOML value as a float
    ///
    /// Panics if the TOML value isn't actually a float
    pub fn num(&self) -> f64 {
        match self {
            Toml::Num(num) => *num,
            _ => panic!(),
        }
    }
    /// Get the TOML value as a string
    ///
    /// Panics if the TOML value isn't actually a string
    pub fn str(&self) -> &str {
        match self {
            Toml::Str(string) => string,
            _ => panic!(),
        }
    }
    /// Get the TOML value as a boolean
    ///
    /// Panics if the TOML value isn't actually a boolean
    pub fn boolean(&self) -> bool {
        match self {
            Toml::Bool(boolean) => *boolean,
            _ => panic!(),
        }
    }
    /// Get the TOML value as a date string
    ///
    /// Panics if the TOML value isn't actually a date string.  See
    /// [the spec](https://toml.io/en/v1.0.0#local-date) for what "date
    /// string" means.
    pub fn date(&self) -> String {
        match self {
            Toml::Date(date) => date.to_string(),
            _ => panic!(),
        }
    }
    /// Get the TOML value as a table
    ///
    /// Panics if the TOML value isn't actually a table
    pub fn arr(&self) -> &Vec<HashMap<String, Toml>> {
        match self {
            Toml::Array(array) => array,
            _ => panic!(),
        }
    }
    /// Get the TOML value as an array
    ///
    /// Panics if the TOML value isn't actually an array
    pub fn simple_arr(&self) -> &Vec<Toml> {
        match self {
            Toml::SimpleArray(array) => array,
            _ => panic!(),
        }
    }
}

/// The error message when failing to parse a TOML string.
#[derive(Clone)]
pub struct TomlErr {
    pub msg: String,
    pub line: usize,
    pub col: usize,
}

impl core::fmt::Debug for TomlErr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Toml error: {}, line:{} col:{}",
            self.msg,
            self.line + 1,
            self.col + 1
        )
    }
}

impl core::fmt::Display for TomlErr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }
}

struct Out {
    out: HashMap<String, Toml>,
    active_array_element: Option<(String, usize)>,
}
impl Out {
    fn start_array(&mut self, key: &str) {
        if !self.out.contains_key(key) {
            self.out.insert(key.to_string(), Toml::Array(vec![]));
        }

        let n = match self.out.get_mut(key).unwrap() {
            Toml::Array(array) => {
                let n = array.len();
                array.push(HashMap::new());
                n
            }
            _ => unreachable!(),
        };

        self.active_array_element = Some((key.to_string(), n));
    }

    fn out(&mut self) -> &mut HashMap<String, Toml> {
        if let Some((table, n)) = self.active_array_element.clone() {
            match self.out.get_mut(&table).unwrap() {
                Toml::Array(array) => &mut array[n],
                _ => unreachable!(),
            }
        } else {
            &mut self.out
        }
    }
}

#[cfg(feature = "no_std")]
impl core::error::Error for TomlErr {}

#[cfg(not(feature = "no_std"))]
impl std::error::Error for TomlErr {}

impl TomlParser {
    /// Parse a TOML string.
    pub fn parse(data: &str) -> Result<HashMap<String, Toml>, TomlErr> {
        let i = &mut data.chars();
        let mut t = TomlParser::default();
        t.next(i);
        let mut out = Out {
            out: HashMap::new(),
            active_array_element: None,
        };
        let mut local_scope = String::new();
        while t.parse_line(i, &mut local_scope, &mut out)? {}

        Ok(out.out)
    }

    fn parse_line(
        &mut self,
        i: &mut Chars,
        local_scope: &mut String,
        out: &mut Out,
    ) -> Result<bool, TomlErr> {
        let tok = self.next_tok(i)?;
        match tok {
            TomlTok::Eof => {
                // at eof.
                return Ok(false);
            }
            TomlTok::BlockOpen => {
                // its a scope
                // we should expect an ident or a string
                let tok = self.next_tok(i)?;
                match tok {
                    TomlTok::Str(key) | TomlTok::Ident(key) => {
                        *local_scope = key;
                        let tok = self.next_tok(i)?;
                        if tok != TomlTok::BlockClose {
                            return Err(self.err_token(tok));
                        }
                    }
                    TomlTok::BlockOpen => {
                        let tok = self.next_tok(i)?;
                        let key = match tok {
                            TomlTok::Ident(key) => key,
                            _ => return Err(self.err_token(tok)),
                        };
                        let tok = self.next_tok(i)?;
                        if tok != TomlTok::BlockClose {
                            return Err(self.err_token(tok));
                        }
                        let tok = self.next_tok(i)?;
                        if tok != TomlTok::BlockClose {
                            return Err(self.err_token(tok));
                        }
                        out.start_array(&key);
                    }
                    _ => return Err(self.err_token(tok)),
                }
            }
            TomlTok::Str(key) | TomlTok::Ident(key) => {
                self.parse_key_value(local_scope, key, i, out.out())?
            }
            _ => return Err(self.err_token(tok)),
        }
        Ok(true)
    }

    fn to_val(&mut self, tok: TomlTok, i: &mut Chars) -> Result<Toml, TomlErr> {
        match tok {
            TomlTok::BlockOpen => {
                let mut vals = Vec::new();
                loop {
                    let tok = self.next_tok(i)?;
                    if tok == TomlTok::BlockClose || tok == TomlTok::Eof {
                        break;
                    }
                    if tok != TomlTok::Comma {
                        vals.push(self.to_val(tok, i)?);
                    }
                }
                Ok(Toml::SimpleArray(vals))
            }
            TomlTok::Str(v) => Ok(Toml::Str(v)),
            TomlTok::U64(v) => Ok(Toml::Num(v as f64)),
            TomlTok::I64(v) => Ok(Toml::Num(v as f64)),
            TomlTok::F64(v) => Ok(Toml::Num(v)),
            TomlTok::Bool(v) => Ok(Toml::Bool(v)),
            TomlTok::Nan(v) => Ok(Toml::Num(if v { -core::f64::NAN } else { core::f64::NAN })),
            TomlTok::Inf(v) => Ok(Toml::Num(if v {
                -core::f64::INFINITY
            } else {
                core::f64::INFINITY
            })),
            TomlTok::Date(v) => Ok(Toml::Date(v)),
            _ => Err(self.err_token(tok)),
        }
    }

    fn parse_key_value(
        &mut self,
        local_scope: &String,
        key: String,
        i: &mut Chars,
        out: &mut HashMap<String, Toml>,
    ) -> Result<(), TomlErr> {
        let tok = self.next_tok(i)?;
        if tok != TomlTok::Equals {
            return Err(self.err_token(tok));
        }
        let tok = self.next_tok(i)?;
        let val = self.to_val(tok, i)?;
        let key = if !local_scope.is_empty() {
            format!("{}.{}", local_scope, key)
        } else {
            key
        };
        out.insert(key, val);
        Ok(())
    }

    fn next(&mut self, i: &mut Chars) {
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

    fn err_token(&self, tok: TomlTok) -> TomlErr {
        TomlErr {
            msg: format!("Unexpected token {:?} ", tok),
            line: self.line,
            col: self.col,
        }
    }

    fn err_parse(&self, what: &str) -> TomlErr {
        TomlErr {
            msg: format!("Cannot parse toml {} ", what),
            line: self.line,
            col: self.col,
        }
    }

    fn next_tok(&mut self, i: &mut Chars) -> Result<TomlTok, TomlErr> {
        while self.cur == '\n' || self.cur == '\r' || self.cur == '\t' || self.cur == ' ' {
            self.next(i);
        }
        loop {
            if self.cur == '\0' {
                return Ok(TomlTok::Eof);
            }

            #[allow(unreachable_patterns)]
            match self.cur as u32 {
                0x2C => {
                    // ,
                    self.next(i);
                    return Ok(TomlTok::Comma);
                }
                0x5B => {
                    // [
                    self.next(i);
                    return Ok(TomlTok::BlockOpen);
                }
                0x5D => {
                    // ]
                    self.next(i);
                    return Ok(TomlTok::BlockClose);
                }
                0x3D => {
                    // =
                    self.next(i);
                    return Ok(TomlTok::Equals);
                }
                0x23 => {
                    // #
                    while self.cur != '\n' && self.cur != '\0' {
                        self.next(i);
                    }

                    while self.cur == '\n'
                        || self.cur == '\r'
                        || self.cur == '\t'
                        || self.cur == ' '
                    {
                        self.next(i);
                    }
                }
                0x2B | 0x2D | 0x30..=0x39 => {
                    // + - 0-9
                    let mut num = String::new();
                    let is_neg = if self.cur == '-' {
                        num.push(self.cur);
                        self.next(i);
                        true
                    } else {
                        if self.cur == '+' {
                            self.next(i);
                        }
                        false
                    };
                    if self.cur == 'n' {
                        self.next(i);
                        if self.cur == 'a' {
                            self.next(i);
                            if self.cur == 'n' {
                                self.next(i);
                                return Ok(TomlTok::Nan(is_neg));
                            } else {
                                return self.parse_bare_key(i, num);
                            }
                        } else {
                            return self.parse_bare_key(i, num);
                        }
                    }
                    if self.cur == 'i' {
                        self.next(i);
                        if self.cur == 'n' {
                            self.next(i);
                            if self.cur == 'f' {
                                self.next(i);
                                return Ok(TomlTok::Inf(is_neg));
                            } else {
                                return self.parse_bare_key(i, num);
                            }
                        } else {
                            return self.parse_bare_key(i, num);
                        }
                    }
                    while self.cur >= '0' && self.cur <= '9' || self.cur == '_' {
                        if self.cur != '_' {
                            num.push(self.cur);
                        }
                        self.next(i);
                    }
                    if self.cur == '.' {
                        num.push(self.cur);
                        self.next(i);
                        while self.cur >= '0' && self.cur <= '9' || self.cur == '_' {
                            if self.cur != '_' {
                                num.push(self.cur);
                            }
                            self.next(i);
                        }
                        if let Ok(num) = num.parse() {
                            return Ok(TomlTok::F64(num));
                        } else {
                            return Err(self.err_parse("number"));
                        }
                    } else if self.cur == '-' {
                        // lets assume its a date. whatever. i don't feel like more parsing today
                        num.push(self.cur);
                        self.next(i);
                        while self.cur >= '0' && self.cur <= '9'
                            || self.cur == ':'
                            || self.cur == '-'
                            || self.cur == 'T'
                        {
                            num.push(self.cur);
                            self.next(i);
                        }
                        return Ok(TomlTok::Date(num));
                    } else {
                        if is_neg {
                            if let Ok(num) = num.parse() {
                                return Ok(TomlTok::I64(num));
                            } else {
                                return self.parse_bare_key(i, num);
                            }
                        }
                        if let Ok(num) = num.parse() {
                            return Ok(TomlTok::U64(num));
                        } else {
                            return self.parse_bare_key(i, num);
                        }
                    }
                }
                0x22 => {
                    // "
                    let mut val = String::new();
                    self.next(i);
                    let mut braces = 1;
                    while self.cur == '"' && braces < 3 {
                        braces += 1;
                        self.next(i);
                    }
                    let escaped_string = braces == 3;
                    loop {
                        if self.cur == '"' && escaped_string == false {
                            break;
                        }
                        if self.cur == '"' && escaped_string {
                            let mut tmp = String::new();
                            let mut braces = 0;
                            while self.cur == '"' {
                                tmp.push('"');
                                braces += 1;
                                self.next(i);
                            }
                            if braces == 3 {
                                break;
                            }
                            val.push_str(&tmp);
                        }
                        if self.cur == '\\' {
                            self.next(i);
                        }
                        if self.cur == '\0' {
                            return Err(self.err_parse("string"));
                        }
                        val.push(self.cur);
                        self.next(i);
                    }
                    self.next(i);
                    return Ok(TomlTok::Str(val));
                }
                bare_key_chars!() => return self.parse_bare_key(i, String::new()),
                _ => return Err(self.err_parse("tokenizer")),
            }
        }
    }

    /// Parse a bare key from the current character.
    fn parse_bare_key(&mut self, i: &mut Chars, mut start: String) -> Result<TomlTok, TomlErr> {
        let mut val = String::new();
        while matches!(self.cur as u32, bare_key_chars!()) {
            val.push(self.cur);
            self.next(i);
        }

        todo!("Return paths for parsing bare key")
    }

    fn next_ident(&mut self, i: &mut Chars, mut start: String) -> Result<TomlTok, TomlErr> {
        while self.cur >= 'a' && self.cur <= 'z'
            || self.cur >= 'A' && self.cur <= 'Z'
            || self.cur == '_'
            || self.cur == '-'
        {
            start.push(self.cur);
            self.next(i);
        }
        if self.cur == '.' {
            while self.cur == '.' {
                self.next(i);
                while self.cur >= 'a' && self.cur <= 'z'
                    || self.cur >= 'A' && self.cur <= 'Z'
                    || self.cur == '_'
                    || self.cur == '-'
                {
                    start.push(self.cur);
                    self.next(i);
                }
            }
            return Ok(TomlTok::Ident(start));
        }
        if start == "true" {
            return Ok(TomlTok::Bool(true));
        }
        if start == "false" {
            return Ok(TomlTok::Bool(false));
        }
        if start == "inf" {
            return Ok(TomlTok::Inf(false));
        }
        if start == "nan" {
            return Ok(TomlTok::Nan(false));
        }
        return Ok(TomlTok::Ident(start));
    }
}
