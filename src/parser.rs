use std::str::CharIndices;

pub struct Parser<'a> {
    str: &'a str,
    chars: CharIndices<'a>,
    pos: Option<usize>,
    chr: Option<char>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Error {
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
        match self.chars.next() {
            Some((pos, ch)) => {
                self.pos = Some(pos);
                self.chr = Some(ch);
                return Some((pos, ch));
            }
            None => return None,
        }
    }
}

pub fn is_whitespace(ch: char) -> bool {
    return ch.is_whitespace() || ch == ',';
}