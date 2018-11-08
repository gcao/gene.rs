use std::str::CharIndices;

use ordered_float::OrderedFloat;
use std::collections::{BTreeMap};

use super::types::Value;
use super::types::Pair;

pub struct Parser<'a> {
    str: &'a str,
    chars: CharIndices<'a>,
    pos: Option<usize>,
    chr: Option<char>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Error<'a> {
    pub message: &'a str,
}

impl<'a> Error<'a> {
    pub fn new(s: &'a str) -> Error<'a> {
        Error {
            message: s,
        }
    }
}

impl<'a> Parser<'a> {
    pub fn new(str: &'a str) -> Parser<'a> {
        Parser {
            str: str,
            chars: str.char_indices(),
            pos: None,
            chr: None,
        }
    }

    pub fn read(&mut self) -> Option<Result<Value, Error>> {
        // Will stop after hitting first non-whitespace char
        self.skip_whitespaces();

        if self.chr.is_none() {
            return Some(Ok(Value::Null));
        }

        let ch = self.chr.unwrap();
        if ch == '[' {
            self.next();
            let mut arr: Vec<Value> = vec![];
            loop {
                self.skip_whitespaces();

                if self.chr.unwrap() == ']' {
                    self.next();
                    break;
                } else {
                    let val = self.read();
                    if !val.is_none() {
                        arr.push(val.unwrap().unwrap());
                    }
                }
            }
            return Some(Ok(Value::Array(arr)));

        } else if ch == '{' {
            self.next();
            let mut map = BTreeMap::new();
            loop {
                self.skip_whitespaces();

                if self.chr.unwrap() == '}' {
                    self.next();
                    break;
                } else {
                    let result = self.read_pair();
                    if !result.is_none() {
                        let pair = result.unwrap().unwrap();
                        map.insert(pair.key, pair.val);
                    }
                }
            }
            return Some(Ok(Value::Map(map)));
        } else if ch == '"' {
            self.next();
            let start = self.pos.unwrap();
            let end = self.advance_while(|ch| ch != '"');
            return Some(Ok(Value::String(self.str[start..end].into())));

        } else if ch == '+' || ch == '-' {
            let next = self.peek();
            if !next.is_none() && next.unwrap().is_digit(10) {
                return self.read_number();
            } else {
                return self.read_keyword_or_symbol();
            }

        } else if ch.is_digit(10) {
            return self.read_number();

        } else if is_symbol_head(ch) {
            return self.read_keyword_or_symbol();

        } else {
            return None;
        }
    }

    pub fn read_number(&mut self) -> Option<Result<Value, Error>> {
        let start = self.pos.unwrap();
        let end = self.advance_while(|ch| !is_whitespace(ch) && !is_sep(ch));
        let s = &self.str[start..end];
        if s.contains('.') {
            let number = s.parse::<f64>().unwrap();
            return Some(Ok(Value::Float(OrderedFloat(number))));
        } else {
            let number = s.parse::<i64>().unwrap();
            return Some(Ok(Value::Integer(number)));
        }
    }

    pub fn read_keyword_or_symbol(&mut self) -> Option<Result<Value, Error>> {
        let mut s = String::from("");
        let word = self.read_word();
        if !word.is_none() {
            s.push_str(&word.unwrap().unwrap());
        }

        match s.as_str() {
            "null"  => Some(Ok(Value::Null)),
            "true"  => Some(Ok(Value::Boolean(true))),
            "false" => Some(Ok(Value::Boolean(false))),
            _       => Some(Ok(Value::Symbol(s)))
        }
    }

    pub fn read_word(&mut self) -> Option<Result<String, Error>> {
        let start =
            if self.pos.is_none() {
                0
            } else {
                self.pos.unwrap()
            };
        let end = self.advance_while(|ch| !is_whitespace(ch));
        return Some(Ok(self.str[start..end].into()));
    }

    pub fn read_pair(&mut self) -> Option<Result<Pair, Error>> {
        if self.chr.unwrap() != '^' {
            return Some(Err(Error::new("Error")));
        } else {
            self.next();
            let key = self.read_word().unwrap().unwrap();
            let val = self.read().unwrap().unwrap();
            return Some(Ok(Pair::new(key, val)));
        }
    }

    fn advance_while<F: FnMut(char) -> bool>(&mut self, mut f: F) -> usize {
        if self.str.len() == 0 {
            return 0;
        }

        // Kick off the iterator
        if self.pos.is_none() {
            self.next();
        }

        loop {
            if f(self.chr.unwrap()) {
                if self.next().is_none() {
                    return self.pos.unwrap() + 1;
                }
            } else {
                return self.pos.unwrap();
            }
        }
    }

    fn next(&mut self) -> Option<(usize, char)> {
        match self.chars.next() {
            Some((pos, ch)) => {
                self.pos = Some(pos);
                self.chr = Some(ch);
                return Some((pos, ch));
            }
            None => return None,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.clone().next().map(|(_, ch)| ch)
    }

    fn skip_whitespaces(&mut self) {
        self.advance_while(is_whitespace);
    }

    // fn reached_end(&mut self) -> bool {
    //     return self.str.len() == 0 ||
    //       (!self.pos.is_none() && self.pos.unwrap() == self.str.len() - 1);
    // }
}

pub fn is_whitespace(ch: char) -> bool {
    return ch.is_whitespace() || ch == ',';
}

pub fn is_sep(ch: char) -> bool {
    match ch {
        '(' |
        ')' |
        '[' |
        ']' |
        '{' |
        '}' => true,
        _   => false,
    }
}

pub fn is_symbol_head(ch: char) -> bool {
    if is_whitespace(ch) {
        return false;
    }

    if ch.is_digit(10) {
        return false;
    }

    match ch {
        '-' |
        '^' |
        '"' |
        '\'' |
        '(' |
        ')' |
        '[' |
        ']' |
        '{' |
        '}' => false,
        _   => true,
    }
}
