mod grammar;
mod lr1_parser;
mod types;

pub use grammar::Grammar;
pub use lr1_parser::{LR1Parser, TreeNode};
pub use types::*;

const DATA_PATH: &str = "./data/";
const ACTION_TABLE: &str = "./data/action_table.rcp";
const GOTO_TABLE: &str = "./data/goto_table.rcp";
const LR1_SETS: &str = "./data/lr1_sets.rcp";

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
