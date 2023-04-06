mod parser;

use crate::parser::{ Point, Token};

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

  for action in &lr1.action_table {
    println!("{:?}", action);
  }

  // 发现问题了，就是在最后的errIndent的位置，出错之后，会回到倒数第三行的位置。
  // 错误处理之后，会继续向后直到遇到能处理的位置，但是后面都没有可以处理的位置了。'
  // 所以会发生越界的问题
  let input: Vec<Token> = vec![
    "'float'", "Ident", "'('", "')'",
    "'{'",
    "'if'", "'('", "IntConst", "')'", "'{'", "IntConst", "';'", "IntConst", "';'", "'}'",
    "'const'", "'int'", "Ident", "'='", "IntConst", "';'",
    "'}'",
    "'const'", "'int'", "Ident", "'='", "'{'", "IntConst", "'}'", "';'",
    "'const'", "'int'", "Ident", "'='", "IntConst", "';'",
    // "'const'", "'int'", "Ident", "'='", "IntConst", "';'",
    // "'const'", "'float'", "Ident", "'='", "FloatConst", "';'",
  ]
    .into_iter()
    .map(|e| Token::new_terminal(e.to_string(), Some(Point::new(0,0))))
    .collect();

  let tmp = lr1.construct_tree(&input);
  print!("{}", tmp);
}