use std::str::CharIndices;

use std::cell::{RefCell, RefMut};
use std::collections::BTreeMap;
use std::rc::Rc;

use ordered_float::OrderedFloat;

use super::types::Gene;
use super::types::Pair;
use super::types::Value;

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
    pub fn new(s: &'a str) -> Self {
        Error { message: s }
    }
}

impl<'a> Parser<'a> {
    pub fn new(str: &'a str) -> Self {
        Parser {
            str,
            chars: str.char_indices(),
            pos: None,
            chr: None,
        }
    }

    pub fn parse(&mut self) -> Result<Value, Error> {
        let mut result = Vec::<Value>::new();

        loop {
            let item = self.read();
            if item.is_none() {
                break;
            } else {
                result.push(item.unwrap().unwrap());
            }
        }

        if result.len() == 1 {
            let first = result[0].clone();
            Ok(first)
        } else {
            Ok(Value::Stream(result))
        }
    }

    pub fn read(&mut self) -> Option<Result<Value, Error>> {
        self.start();

        // Will stop after hitting first non-whitespace char
        self.skip_whitespaces();

        let ch = self.chr?;
        if ch == '(' {
            self.next();
            let mut type_is_set = false;
            let mut _type = Value::Null;
            let mut props = BTreeMap::<String, Rc<RefCell<Value>>>::new();
            let mut data = Vec::<Rc<RefCell<Value>>>::new();
            loop {
                self.skip_whitespaces();

                if self.chr.unwrap() == ')' {
                    self.next();
                    break;
                } else if self.chr.unwrap() == '^' {
                    let result = self.read_pair();
                    if result.is_some() {
                        let pair = result.unwrap().unwrap();
                        props.insert(pair.key, Rc::new(RefCell::new(pair.val)));
                    }
                } else {
                    let result = self.read();
                    if result.is_some() {
                        let val = result.unwrap().unwrap();
                        if type_is_set {
                            data.push(Rc::new(RefCell::new(val)));
                        } else {
                            type_is_set = true;
                            _type = val;
                        }
                    }
                }
            }
            return Some(Ok(Value::Gene(Gene {
                _type: Rc::new(RefCell::new(_type)),
                props,
                data,
            })));
        } else if ch == '[' {
            self.next();
            let mut arr: Vec<Value> = vec![];
            loop {
                self.skip_whitespaces();

                if self.chr.unwrap() == ']' {
                    self.next();
                    break;
                } else {
                    let val = self.read();
                    if val.is_some() {
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
                    if result.is_some() {
                        let pair = result.unwrap().unwrap();
                        map.insert(pair.key, pair.val);
                    }
                }
            }
            return Some(Ok(Value::Map(map)));
        } else if ch == '"' {
            self.next();
            return self.read_string();
        } else if ch == '#' {
            let next_ch = self.peek().unwrap();
            if is_whitespace(next_ch) || next_ch == '!' {
                self.next();
                self.advance_while(|ch| ch != '\n');
                return self.read();
            } else {
                return Some(Ok(Value::Symbol(self.read_word().unwrap().unwrap())));
            }
        } else if ch == '+' || ch == '-' {
            let next = self.peek();
            if next.is_some() && next.unwrap().is_digit(10) {
                return self.read_number();
            } else {
                return self.read_keyword_or_symbol();
            }
        } else if ch.is_digit(10) {
            return self.read_number();
        } else if ch == '`' {
            self.next();
            let mut gene = Gene::new(Value::Symbol("#QUOTE".to_string()));
            gene.data
                .push(Rc::new(RefCell::new(self.read().unwrap().unwrap())));
            return Some(Ok(Value::Gene(gene)));
        } else if is_symbol_head(ch) {
            return self.read_keyword_or_symbol();
        } else {
            return None;
        }
    }

    fn read_number(&mut self) -> Option<Result<Value, Error>> {
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

    fn read_string(&mut self) -> Option<Result<Value, Error>> {
        let mut result = String::from("");

        let mut escaped = false;

        loop {
            let ch = self.chr.unwrap();
            if ch == '\\' {
                // Do not treat whitespace, ()[]{} etc as special char
                escaped = true;
            } else if escaped {
                escaped = false;
                result.push(ch);
            } else if ch == '"' {
                self.next();
                break;
            } else {
                result.push(ch);
            }

            // Move forward
            if self.next().is_none() {
                break;
            }
        }

        Some(Ok(Value::String(result.to_string())))
    }

    fn read_keyword_or_symbol(&mut self) -> Option<Result<Value, Error>> {
        let is_escape = self.chr.unwrap() == '\\';

        let mut s = String::from("");
        let word = self.read_word();
        if word.is_some() {
            s.push_str(&word.unwrap().unwrap());
        }

        if is_escape {
            return Some(Ok(Value::Symbol(s)));
        }

        match s.as_str() {
            "null" => Some(Ok(Value::Null)),
            "true" => Some(Ok(Value::Boolean(true))),
            "false" => Some(Ok(Value::Boolean(false))),
            _ => Some(Ok(Value::Symbol(s))),
        }
    }

    fn read_word(&mut self) -> Option<Result<String, Error>> {
        let mut result = String::from("");

        let mut escaped = false;

        loop {
            let ch = self.chr.unwrap();
            if ch == '\\' {
                // Do not treat whitespace, ()[]{} etc as special char
                escaped = true;
            } else if escaped {
                escaped = false;
                result.push(ch);
            } else if is_whitespace(ch) || is_sep(ch) {
                break;
            } else {
                result.push(ch);
            }

            // Move forward
            if self.next().is_none() {
                break;
            }
        }

        Some(Ok(result.to_string()))
    }

    fn read_pair(&mut self) -> Option<Result<Pair, Error>> {
        if self.chr.unwrap() != '^' {
            return Some(Err(Error::new("Error")));
        } else {
            self.next();
            let ch = self.chr.unwrap();
            if ch == '^' {
                self.next();
                let key = self.read_word().unwrap().unwrap();
                let val = Value::Boolean(true);
                return Some(Ok(Pair::new(key, val)));
            } else if ch == '!' {
                self.next();
                let key = self.read_word().unwrap().unwrap();
                let val = Value::Boolean(false);
                return Some(Ok(Pair::new(key, val)));
            } else {
                let key = self.read_word().unwrap().unwrap();
                let val = self.read().unwrap().unwrap();
                return Some(Ok(Pair::new(key, val)));
            }
        }
    }

    /// Return the index of next char or str.len()
    fn advance_while<F: FnMut(char) -> bool>(&mut self, mut f: F) -> usize {
        loop {
            if self.chr.is_none() {
                return self.str.len();
            } else if f(self.chr.unwrap()) {
                self.next();
            } else {
                return self.pos.unwrap();
            }
        }
    }

    fn start(&mut self) {
        if self.pos.is_none() {
            self.next();
        }
    }

    fn next(&mut self) -> Option<(usize, char)> {
        match self.chars.next() {
            Some((pos, ch)) => {
                self.pos = Some(pos);
                self.chr = Some(ch);
                Some((pos, ch))
            }
            None => {
                self.pos = Some(self.str.len());
                self.chr = None;
                None
            }
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.clone().next().map(|(_, ch)| ch)
    }

    fn skip_whitespaces(&mut self) {
        self.advance_while(is_whitespace);
    }

    // fn reached_end(&mut self) -> bool {
    //     self.str.len() == 0 || (self.pos.is_some() && self.pos.unwrap() == self.str.len() - 1)
    // }
}

pub fn is_whitespace(ch: char) -> bool {
    ch.is_whitespace() || ch == ','
}

pub fn is_sep(ch: char) -> bool {
    match ch {
        '(' | ')' | '[' | ']' | '{' | '}' => true,
        _ => false,
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
        '^' | '"' | '\'' | '(' | ')' | '[' | ']' | '{' | '}' => false,
        _ => true,
    }
}
