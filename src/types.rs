#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Value {
    Todo,
    Null,
    Boolean(bool),
    Symbol(String),
}
