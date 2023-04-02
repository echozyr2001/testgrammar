mod parser;

use crate::parser::{Element, TreeNode};


fn main() {
  let path = "./g2.txt";
  let mut grammar = parser::Grammar::new();
  grammar.grammar_load(path);

  let mut lr1 = parser::LR1Parser::new();
  lr1.compute_lr1_item_sets(&grammar);
  lr1.construct_parsing_table(&grammar);
  // println!("Action Table:");
  // for ((state, symbol), action) in &lr1.action_table {
  //   println!("({}, {:?}) -> {:?}", state, symbol, action);
  // }
  //
  // println!("\nGoto Table:");
  // for ((state, symbol), next_state) in &lr1.goto_table {
  //   println!("({}, {:?}) -> {}", state, symbol, next_state);
  // }
  // let mut sum = 0;
  // for i in &lr1.lr1_sets {
  //   sum += i.len();
  // }
  // println!("sum: {}", sum);
  // println!("{}", lr1.lr1_sets.len());
  // for item_set in &lr1.lr1_sets {
  //   for item in item_set {
  //     if item.head == Element::NotTerminal("F".to_string()) {
  //       println!("{:?}", item.lookahead);
  //     }
  //   }
  // }

  // 设置测试用例
  // let input_str = "aed";
  // let mut input = Vec::<Element>::new();
  // for c in input_str.chars() {
  //   input.push(Element::Terminal(c.to_string()));
  // }
  // let tmp = lr1.construct_tree(&action_table, &goto_table, &input).unwrap();
  // // 分别输出tmp中的内容
  // print_tree(&tmp, 0);

  let mut input = Vec::<Element>::new();
  input.push(Element::Terminal("'const'".to_string()));
  input.push(Element::Terminal("'int'".to_string()));
  input.push(Element::Terminal("Ident".to_string()));
  input.push(Element::Terminal("'='".to_string()));
  // input.push(Element::Terminal("'='".to_string()));
  input.push(Element::Terminal("IntConst".to_string()));
  input.push(Element::Terminal("';'".to_string()));

  // input.push(Element::Terminal("'const'".to_string()));
  // input.push(Element::Terminal("'float'".to_string()));
  // input.push(Element::Terminal("Ident".to_string()));
  // input.push(Element::Terminal("'='".to_string()));
  // // input.push(Element::Terminal("FloatConst".to_string()));
  // input.push(Element::Terminal("';'".to_string()));
  let tmp = lr1.construct_tree(&input);
  match tmp {
    Ok(ok) => {
      print_tree(&ok, 0);
    }
    Err(err) => {
      println!("err :{:#?}",err);
    }
  }
}

// 递归输出语法树, 使用depth来控制缩进
fn print_tree(tree: &TreeNode, depth: usize) {
  let mut indent = String::new();
  for _ in 0..depth {
    indent.push_str("  ");
  }
  if tree.element != Element::Terminal("ε".to_string()) {
    println!("{}{:?}", indent, tree.element);
  }
  if let Some(children) = &tree.children {
    for child in children {
      print_tree(child, depth + 1);
    }
  }
}