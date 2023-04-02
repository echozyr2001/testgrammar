use serde::{Serialize, Deserialize};

#[derive(Debug, Eq, Hash, PartialEq, Clone, Serialize, Deserialize)]
pub enum Element {
  Terminal(String),
  NotTerminal(String),
}

pub(crate) type Item = Vec<Element>;
pub(crate) type PHead = Element;
pub(crate) type PBody = Vec<Item>;