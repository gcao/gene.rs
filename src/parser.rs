use std::collections::BTreeMap;
use std::str::CharIndices;

use ordered_float::OrderedFloat;

use super::Value;
use super::Gene;

pub struct Parser<'a> {
    str: &'a str,
    chars: CharIndices<'a>,
    pos: Option<usize>,
    chr: Option<char>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Error {
    pub lo: usize,
    pub hi: usize,
    pub message: String,
}

pub struct Pair {
    pub key: String,
    pub val: Value,
}
// pub enum Parsed {
//     Unparsed,
//     Value(Value)
// }

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
        self.whitespace();

        self.chars.clone().next().map(|(pos, ch)| match (pos, ch) {
            (start, '0'...'9') => {
                let end = self.advance_while(|ch| ch.is_digit(10));
                if self.peek() == Some('.') {
                    self.next();
                    let end = self.advance_while(|ch| ch.is_digit(10));
                    Ok(Value::Float(OrderedFloat(
                        self.str[start..end].parse().unwrap(),
                    )))
                } else {
                    Ok(Value::Integer(self.str[start..end].parse().unwrap()))
                }
            }
            (start, ch @ '+') | (start, ch @ '-') => {
                self.next();
                match self.peek() {
                    Some('0'...'9') => {
                        let start = if ch == '+' { start + 1 } else { start };
                        let end = self.advance_while(|ch| ch.is_digit(10));
                        if self.peek() == Some('.') {
                            self.next();
                            let end = self.advance_while(|ch| ch.is_digit(10));
                            Ok(Value::Float(OrderedFloat(
                                self.str[start..end].parse().unwrap(),
                            )))
                        } else {
                            Ok(Value::Integer(self.str[start..end].parse().unwrap()))
                        }
                    }
                    Some(ch) if is_symbol_tail(ch) => {
                        let end = self.advance_while(is_symbol_tail);
                        Ok(Value::Symbol(self.str[start..end].into()))
                    }
                    None | Some(' ') | Some('\t') | Some('\n') => Ok(Value::Symbol(ch.to_string())),
                    _ => unimplemented!(),
                }
            }
            (start, '.') => {
                self.next();
                if let Some('0'...'9') = self.peek() {
                    let end = self.advance_while(|ch| ch.is_digit(10));
                    Ok(Value::Float(OrderedFloat(
                        self.str[start..end].parse().unwrap(),
                    )))
                } else {
                    let end = self.advance_while(is_symbol_tail);
                    Ok(Value::Symbol(self.str[start..end].into()))
                }
            }
            (start, '\\') => {
                self.next();
                let start = start + 1;
                let end = self.advance_while(|ch| !ch.is_whitespace());
                Ok(Value::Char(match &self.str[start..end] {
                    "newline" => '\n',
                    "return" => '\r',
                    "space" => ' ',
                    "tab" => '\t',
                    otherwise => {
                        if otherwise.chars().count() == 1 {
                            otherwise.chars().next().unwrap()
                        } else {
                            return Err(Error {
                                lo: start - 1,
                                hi: end,
                                message: format!("invalid char literal `\\{}`", otherwise),
                            });
                        }
                    }
                }))
            }
            (start, '"') => {
                self.next();
                let mut string = String::new();
                loop {
                    match self.next() {
                        Some((_, '"')) => return Ok(Value::String(string)),
                        Some((_, '\\')) => {
                            string.push(match self.next() {
                                Some((_, 't')) => '\t',
                                Some((_, 'r')) => '\r',
                                Some((_, 'n')) => '\n',
                                Some((_, '\\')) => '\\',
                                Some((_, '"')) => '\"',
                                Some((pos, ch)) => {
                                    return Err(Error {
                                        lo: pos - 1,
                                        hi: pos + 1,
                                        message: format!("invalid string escape `\\{}`", ch),
                                    })
                                }
                                None => unimplemented!(),
                            });
                        }
                        Some((_, ch)) => string.push(ch),
                        None => {
                            return Err(Error {
                                lo: start,
                                hi: self.str.len(),
                                message: "expected closing `\"`, found EOF".into(),
                            })
                        }
                    }
                }
            }
            (start, open @ '(') | (start, open @ '[') => {
                let close = match open {
                    '(' => ')',
                    '[' => ']',
                    _ => unreachable!(),
                };

                self.next();
                let mut items = vec![];
                loop {
                    self.whitespace();

                    if self.peek() == Some(close) {
                        self.next();
                        return Ok(match open {
                            // '(' => Value::Gene(items),
                            '[' => Value::Vector(items),
                            _ => unreachable!(),
                        });
                    }

                    match self.read() {
                        Some(Ok(value)) => items.push(value),
                        Some(Err(err)) => return Err(err),
                        None => {
                            return Err(Error {
                                lo: start,
                                hi: self.str.len(),
                                message: format!("unclosed `{}`", open),
                            })
                        }
                    }
                }
            }
            (start, open @ '{') => {
                self.next();
                let close = '}';
                let mut items = vec![];
                loop {
                    self.whitespace();

                    if self.peek() == Some(close) {
                        self.next();
                        let mut map = BTreeMap::new();
                        let mut iter = items.into_iter();
                        while let Some(key) = iter.next() {
                            if let Some(value) = iter.next() {
                                map.insert(key, value);
                            } else {
                                let end = self.chars
                                    .clone()
                                    .next()
                                    .map(|(pos, _)| pos)
                                    .unwrap_or(self.str.len());
                                return Err(Error {
                                    lo: start,
                                    hi: end,
                                    message: "odd number of items in a Map".into(),
                                });
                            }
                        }
                        return Ok(Value::Map(map));
                    }

                    match self.read() {
                        Some(Ok(value)) => items.push(value),
                        Some(Err(err)) => return Err(err),
                        None => {
                            return Err(Error {
                                lo: start,
                                hi: self.str.len(),
                                message: format!("unclosed `{}`", open),
                            })
                        }
                    }
                }
            }
            (start, ch) if is_symbol_head(ch) => {
                self.next();
                let end = self.advance_while(is_symbol_tail);
                Ok(match &self.str[start..end] {
                    "true" => Value::Boolean(true),
                    "false" => Value::Boolean(false),
                    "null" => Value::Null,
                    otherwise => Value::Symbol(otherwise.into()),
                })
            }
            (_, '/') => {
                self.next();
                Ok(Value::Symbol("/".into()))
            }
            _ => unimplemented!(),
        })
    }

    pub fn read_pair(&mut self) -> Option<Result<Pair, Error>> {
        self.whitespace();

        if self.peek() == Some('^') {
            self.next();
            let key: String;
            let val: Value;

            if self.peek() == Some('^') {
                // ^^key
                unimplemented!()
            } else if self.peek() == Some('!') {
                // ^!key
                unimplemented!()
            } else {
                // ^key value
                self.chars.clone().next().map(|(pos, ch)| match (pos, ch) {
                    (start, '^') => {
                        unimplemented!()
                    }
                    _ => unimplemented!()
                })
            }
        } else {
            unimplemented!()
        }
    }

    // Read word till whitespace or ,)]} or end of input
    pub fn read_word(&mut self, start: usize) -> Option<Result<String, Error>> {
        let end = self.advance_while(|ch| ch.is_whitespace() || ch == ',');
        return Some(Ok(self.str[start..end].into()));
    }

    // fn read_string(&mut self, c: char) -> Parsed {
    //     match c {
    //         '"' => {
    //             return Parsed::Value(Value::String(r"".to_string()))
    //         }
    //         _ => {
    //             return Parsed::Unparsed
    //         }
    //     }
    // }

    fn peek(&self) -> Option<char> {
        self.chars.clone().next().map(|(_, ch)| ch)
    }

    fn whitespace(&mut self) {
        loop {
            // Skip whitespace.
            self.advance_while(|ch| ch.is_whitespace() || ch == ',');
            // Skip comment if present.
            if self.chars.clone().next().map_or(false, |(_, ch)| ch == ';') {
                self.advance_while(|ch| ch != '\n');
                self.next();
            } else {
                // Otherwise we're done.
                return;
            }
        }
    }

    fn advance_while<F: FnMut(char) -> bool>(&mut self, mut f: F) -> usize {
        loop {
            match self.chars.clone().next() {
                Some((pos, ch)) => {
                    if f(ch) {
                        self.next();
                    } else {
                        return pos;
                    }
                }
                None => return self.str.len(),
            }
        }
    }

    fn next(&mut self) -> Option<(usize, char)> {
        match self.next() {
            Some((pos, ch)) => {
                self.pos = Some(pos);
                self.chr = Some(ch);
                return Some((pos, ch));
            }
            None => return
        }
    }
}

fn is_symbol_head(ch: char) -> bool {
    match ch {
        ' ' |
        '"' |
        '\'' |
        '(' |
        ')' |
        '[' |
        ']' |
        '{' |
        '}' |
        ',' => false,
        _   => true,
    }
}

fn is_symbol_tail(ch: char) -> bool {
    is_symbol_head(ch)
}
