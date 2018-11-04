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
        return Some(Ok("".into()));
    }
}
