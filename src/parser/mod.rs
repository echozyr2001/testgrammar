mod grammar;
mod lr1_parser;
mod types;

use lazy_static::lazy_static;
pub use grammar::Grammar;
pub use lr1_parser::{LR1Parser, TreeNode};
pub use types::*;

const DATA_PATH: &str = "./data/";
const ACTION_TABLE: &str = "./data/action_table.rcp";
const GOTO_TABLE: &str = "./data/goto_table.rcp";
const LR1_SETS: &str = "./data/lr1_sets.rcp";

lazy_static!(
  pub static ref START_SYMBOL: Token = Token::new_not_terminal("CompUnit'".to_string(), None);
  pub static ref END_SYMBOL: Token = Token::new_terminal("#".to_string(), None);
  pub static ref EMPTY_SYMBOL: Token = Token::new_terminal("ε".to_string(), None);
  pub static ref ERROR_SYMBOL: Token = Token::new_terminal("err".to_string(), None);
);

// const END_SYMBOL: Token = Token::new_terminal("#".to_string(), Point::new(0, 0));

#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    let left = crate::parser::Element::NotTerminal("'{'".to_string());
    let right = crate::parser::Element::NotTerminal("'}'".to_string());
    assert!(left.is_paired(&right));
    let left = crate::parser::Element::NotTerminal("'('".to_string());
    let right = crate::parser::Element::NotTerminal("')'".to_string());
    assert!(left.is_paired(&right));
    let left = crate::parser::Element::NotTerminal("'['".to_string());
    let right = crate::parser::Element::NotTerminal("']'".to_string());
    assert!(left.is_paired(&right));
  }
}
