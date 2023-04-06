use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Clone, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Token {
  token_type: TokenType,
  value: String,
  pos: Option<Point>,
}

impl Token {
  fn new(token_type: TokenType, value: String, pos: Option<Point>) -> Self {
    Self {
      token_type,
      value,
      pos,
    }
  }

  pub fn new_terminal(value: String, pos: Option<Point>) -> Self {
    Self::new(TokenType::Terminal(TokenKind::Identifier), value, pos)
  }

  pub fn new_not_terminal(value: String, pos: Option<Point>) -> Self {
    Self::new(TokenType::NotTerminal(TokenKind::Identifier), value, pos)
  }

  pub fn is_terminal(&self) -> bool {
    if let TokenType::Terminal(_) = self.token_type {
      return true;
    }
    false
  }

  pub fn is_not_terminal(&self) -> bool {
    if let TokenType::NotTerminal(_) = self.token_type {
      return true;
    }
    false
  }

  pub fn get_pos(&self) -> &Option<Point> {
    &self.pos
  }

  pub fn get_value(&self) -> &String {
    &self.value
  }
}

#[derive(Debug, Eq, PartialEq, Clone, Ord, PartialOrd, Serialize, Deserialize)]
pub enum TokenType {
  Terminal(TokenKind),
  NotTerminal(TokenKind),
}

#[derive(Debug, Eq, PartialEq, Clone, Ord, PartialOrd, Serialize, Deserialize)]
pub enum TokenKind {
  Keyword,
  Identifier,
  Operator,
  Literal,
  Separator,
}

#[derive(Debug, Eq, PartialEq, Clone,Copy, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Point {
  row: usize,
  col: usize,
}

impl Point {
  pub fn new(row: usize, col: usize) -> Self {
    Self {
      row,
      col,
    }
  }
}

pub(crate) type Item = Vec<Token>;
pub(crate) type PHead = Token;
pub(crate) type PBody = Vec<Item>;
