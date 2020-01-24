use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Ipld {
    Null,
    Bool(bool),
    Integer(i128),
    Float(f64),
    String(String),
    Bytes(Vec<u8>),
    List(Vec<Ipld>),
    Map(BTreeMap<String, Ipld>),
    Link(Vec<u8>),
}
