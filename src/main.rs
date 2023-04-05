mod parser;

use crate::parser::Element;

fn main() {
  let path = "./g3.txt";
  let mut grammar = parser::Grammar::new();
  grammar.grammar_load(path);
  // println!("{:?}", grammar.start_symbol);
  // println!("{:?}", grammar.token_list);
  // println!("{:?}", grammar.pro_list);
  // println!("{:?}", grammar.first_sets);

  let mut lr1 = parser::LR1Parser::new();
  lr1.compute_lr1_item_sets(&grammar);
  lr1.construct_parsing_table(&grammar);
  //
  // // for action in &lr1.action_table {
  // //   println!("{:?}", action);
  // // }
  //
  let input: Vec<Element> = vec![
    "'float'", "Ident", "'('", "')'",
    "'{'",
    "'if'", "'('", "errIntConst", "')'", "'{'", "errIntConst", "';'", "IntConst", "';'", "'}'",
    "'const'", "'int'", "Ident", "'='", "IntConst", "';'",
    "'}'",
    "'const'", "'int'", "Ident", "'='", "'{'", "errIntConst", "'}'", "';'",
    "'const'", "'int'", "Ident", "'='", "IntConst", "';'",
    // "'const'", "'int'", "Ident", "'='", "IntConst", "';'",
    // "'const'", "'float'", "Ident", "'='", "FloatConst", "';'",
  ]
    .into_iter()
    .map(|e| Element::Terminal(e.to_string()))
    .collect();

  let tmp = lr1.construct_tree(&input);
  print!("{}", tmp);
}