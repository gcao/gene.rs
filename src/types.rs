use std::collections::BTreeMap;

use super::Value;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Gene {
  _type: Box<Value>,
  properties: Option<BTreeMap<String, Value>>,
  data: Option<Vec<Value>>,
}

impl Gene {
  pub fn new(
    _type: Value,
    properties: Option<BTreeMap<String, Value>>,
    data: Option<Vec<Value>>,
  ) -> Gene {
    Gene {
      _type: Box::new(_type),
      properties: properties,
      data: data,
    }
  }
}