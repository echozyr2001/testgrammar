use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, Hash, PartialEq, Clone, Serialize, Deserialize)]
pub enum Element {
    Terminal(String),
    NotTerminal(String),
}
impl Element {
    // pub(crate) fn is_semicolon(&self) -> bool {
    //     if let Element::Terminal(token) = self {
    //         return token.as_str() == "';'";
    //     }
    //     false
    // }
    // pub(crate) fn is_left_bracket(&self) -> bool {
    //     if let Element::Terminal(token) = self {
    //         return token.as_str() == "'{'" || token.as_str() == "'['" || token.as_str() == "'('";
    //     }
    //     false
    // }
    // pub(crate) fn is_right_bracket(&self) -> bool {
    //     if let Element::Terminal(token) = self {
    //         return token.as_str() == "'}'" || token.as_str() == "']'" || token.as_str() == "')'";
    //     }
    //     false
    // }
    // pub(crate) fn is_paired(&self, rhs: &Element) -> bool {
    //     let left = self.clone().unwarp().0;
    //     let right = rhs.clone().unwarp().0;
    //     match right.as_str() {
    //         "'}'" => left.as_str() == "'{'",
    //         "']'" => left.as_str() == "'['",
    //         "')'" => left.as_str() == "'('",
    //         &_ => false,
    //     }
    // }

    pub(crate) fn unwarp(self) -> (String, usize) {
        match self {
            Self::NotTerminal(v) => (v, 0),
            Self::Terminal(v) => (v, 1),
        }
    }
    pub(crate) fn get_pos(&self) -> usize {
        0
    }
}

pub(crate) type Item = Vec<Element>;
pub(crate) type PHead = Element;
pub(crate) type PBody = Vec<Item>;
