use std::collections::{HashMap, HashSet};
use std::fs::{create_dir_all, File, metadata};
use std::hash::Hash;
use serde::{Serialize, Deserialize};
use crate::parser::{ACTION_TABLE, DATA_PATH, GOTO_TABLE, Grammar, LR1_SETS};
use crate::parser::types::Element;

pub struct LR1Parser {
  pub lr1_sets: LR1Sets,
  pub action_table: ActionTable,
  pub goto_table: GotoTable,
}

#[derive(Debug)]
pub struct ParserError {
  pub error_type: ErrorType,
  pub error_pos: (usize, usize),
}

#[derive(Debug)]
pub enum ErrorType {
  // 缺失错误
  MissingError(String),
  // 多余错误
  ExtraError(String),
  // 未知错误
  UnknownError(String),
}

#[derive(Clone, Hash, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct LR1Item {
  pub(crate) head: Element,
  body: Vec<Element>,
  dot: usize,
  pub(crate) lookahead: Element,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
  Shift(usize),
  Reduce(Element, Vec<Element>),
  Accept,
}

type State = usize;
type GotoTable = HashMap<(State, Element), State>;
type ActionTable = HashMap<(State, Element), Action>;
type LR1Sets = Vec<HashSet<LR1Item>>;
type ErrorList = Vec<ParserError>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TreeNode {
  pub element: Element,
  pub children: Option<Vec<TreeNode>>,
}

fn file_exists(file_path: &str) -> bool {
  match metadata(file_path) {
    Ok(metadata) => metadata.is_file(),
    Err(_) => false,
  }
}

impl LR1Parser {
  pub fn new() -> Self {
    Self {
      lr1_sets: LR1Sets::new(),
      action_table: ActionTable::new(),
      goto_table: GotoTable::new(),
    }
  }

  pub fn construct_tree(&self, input: &Vec<Element>) -> Result<TreeNode, ErrorList> {
    let mut state_stack: Vec<State> = Vec::new();
    let mut node_stack: Vec<TreeNode> = Vec::new();
    let mut error_list = ErrorList::new();

    let mut input_buff = input.clone();
    input_buff.push(Element::Terminal("#".to_string()));

    state_stack.push(0);
    node_stack.push(TreeNode { element: Element::NotTerminal("#".to_string()), children: None });

    let mut input_pos = 0;

    let mut err = false;
    loop {
      if input_pos >= input_buff.len() {
        break;
      }
      let state = state_stack.last().unwrap().clone();
      let symbol = input_buff[input_pos].clone();
      let action = self.action_table.get(&(state.clone(), symbol.clone())).cloned();

      match action {
        Some(Action::Shift(state)) => {
          state_stack.push(state);
          node_stack.push(TreeNode { element: symbol.clone(), children: None });
          input_pos += 1;
        }
        Some(Action::Reduce(prod_head, prod_body)) => {
          let mut children: Vec<TreeNode> = Vec::new();
          for _ in 0..prod_body.len() {
            state_stack.pop();
            children.push(node_stack.pop().unwrap());
          }
          children.reverse();

          let state = state_stack.last().unwrap().clone();
          let state = self.goto_table.get(&(state, prod_head.clone())).unwrap().clone();

          state_stack.push(state);
          node_stack.push(TreeNode { element: prod_head, children: Some(children) });
        }
        Some(Action::Accept) => {
          break;
        }
        None => {
          if let Some(Action::Shift(t)) = self.action_table.get(&(state.clone(), Element::Terminal("ε".to_string()))).cloned() {
            state_stack.push(t);
            node_stack.push(TreeNode { element: Element::Terminal("ε".to_string()), children: None });
          } else {
            err = true;
            // error_list.(format!("Unexpected symbol '{:?}' at position {}", symbol, input_pos));
            error_list.push(ParserError {
              error_type: ErrorType::UnknownError(format!("Unexpected symbol '{:?}' at position {}", symbol, input_pos)),
              error_pos: (input_pos, input_pos + 1),
            });

            // 错误处理：跳过输入直到找到一个可以接受的符号
            let mut found_acceptable_symbol = false;
            while !found_acceptable_symbol {
              input_pos += 1;
              if input_pos >= input_buff.len() {
                break;
              }
              let next_symbol = input_buff[input_pos].clone();
              if let Some(_) = self.action_table.get(&(state.clone(), next_symbol.clone())) {
                found_acceptable_symbol = true;
              }
            }
          }
          // match empty_action {
          //   Some(Action::Shift(t)) => {
          //     state_stack.push(*t);
          //     node_stack.push(TreeNode { element: Element::Terminal("ε".to_string()), children: None });
          //   }
          //   _ => {
          //     err = true;
          //     // 错误处理
          //     // 静默错误恢复，跳过错误的token
          //     error_list.push(ParserError { error_type: ErrorType::UnknownError(format!("Unknown Error")), error_pos: (0, 0) });
          //     let mut state = state_stack.last().unwrap();
          //     let mut symbol = input_buff.get(input_pos).unwrap();
          //
          //     while self.action_table.get(&(state.clone(), symbol.clone())) == None && input_pos < input_buff.len() - 1 {
          //       input_pos += 1;
          //       state = state_stack.last().unwrap();
          //       symbol = input_buff.get(input_pos).unwrap();
          //     }
          //
          //     // 预测性错误恢复
          //     // let mut except_symbol = Vec::<Element>::new();
          //     // for action in &self.action_table {
          //     //   if action.0.0 == *state {
          //     //     except_symbol.push(action.0.1.clone());
          //     //   }
          //     // }
          //     // input_buff.insert(input_pos, except_symbol.last().unwrap().clone());
          //     // error_list.push(ParserError { error_type: ErrorType::MissingError(format!("Missing {:?}", except_symbol)), error_pos: (0, 0) });
          //
          //     // 突发式错误恢复，回溯到最近的一个可恢复的状态
          //     // loop {
          //     //   let state = state_stack.pop().unwrap();
          //     //   node_stack.pop();
          //     //
          //     //   if self.action_table.get(&(state.clone(), symbol.clone())) != None {
          //     //     break;
          //     //   }
          //     // }
          //
          //     // 面向错误的恢复，插入、删除、替换
          //     // 迭代错误恢复
          //     // iterative_error_recovery(&self.action_table, &self.goto_table, &mut state_stack, &mut node_stack, &mut input_buff, &mut input_pos);
          //   }
          // }
        }
      }
    }
    return if err {
      Err(error_list)
    } else {
      Ok(node_stack.pop().unwrap())
    };
  }

  pub fn construct_parsing_table(&mut self, grammar: &Grammar) {
    if file_exists(ACTION_TABLE) && file_exists(GOTO_TABLE) {
      let action_file = File::open(ACTION_TABLE).expect("Unable to open action table file");
      let goto_file = File::open(GOTO_TABLE).expect("Unable to open goto table file");
      self.action_table = bincode::deserialize_from(action_file).unwrap();
      self.goto_table = bincode::deserialize_from(goto_file).unwrap();
    } else {
      self.construct_parsing_table_core(grammar);

      create_dir_all(DATA_PATH).expect("Unable to create action table file");
      let mut action_file = File::create(ACTION_TABLE).expect("Unable to create action table file");
      let mut goto_file = File::create(GOTO_TABLE).expect("Unable to create goto table file");
      bincode::serialize_into(&mut action_file, &self.action_table).expect("Unable to serialize action table");
      bincode::serialize_into(&mut goto_file, &self.goto_table).expect("Unable to serialize goto table");
    }
  }

  // 构建LR1分析表
  fn construct_parsing_table_core(&mut self, grammar: &Grammar) {
    for (state, item_set) in self.lr1_sets.iter().enumerate() {
      for item in item_set {
        let dot_position = item.dot;
        let next_symbol = item.body.get(dot_position);
        match next_symbol {
          Some(Element::Terminal(a)) => {
            let goto_set = self.goto(grammar, item_set, &Element::Terminal(a.to_string()));
            let goto_state = self.lr1_sets.iter().position(|x| *x == goto_set).unwrap();
            self.action_table.insert((state, Element::Terminal(a.to_string())), Action::Shift(goto_state));
          }
          Some(Element::NotTerminal(a)) => {
            let goto_set = self.goto(grammar, item_set, &Element::NotTerminal(a.to_string()));
            let goto_state = self.lr1_sets.iter().position(|x| *x == goto_set).unwrap();
            self.goto_table.insert((state, Element::NotTerminal(a.to_string())), goto_state);
          }
          None => {
            if item.head == grammar.start_symbol && item.lookahead == Element::Terminal("#".to_string()) {
              self.action_table.insert((state, Element::Terminal("#".to_string())), Action::Accept);
            } else {
              let prod_index = grammar.pro_list.get(&item.head).unwrap().iter().position(|x| *x == item.body).unwrap();
              self.action_table.insert((state, item.lookahead.clone()), Action::Reduce(item.head.clone(), grammar.pro_list.get(&item.head).unwrap()[prod_index].clone()));
            }
          }
        }
      }
    }
  }

  pub fn compute_lr1_item_sets(&mut self, grammar: &Grammar) {
    if file_exists(LR1_SETS) {
      let lr1_file = File::open(LR1_SETS).expect("Unable to open action table file");
      self.lr1_sets = bincode::deserialize_from(lr1_file).unwrap();
    } else {
      self.compute_lr1_item_sets_core(grammar, &grammar.start_symbol);

      create_dir_all(DATA_PATH).expect("Unable to create action table file");
      let mut lr1_file = File::create(LR1_SETS).expect("Unable to create action table file");
      bincode::serialize_into(&mut lr1_file, &self.lr1_sets).expect("Unable to serialize action table");
    }
  }

  // NOTE: 不使用queue和visited
  fn compute_lr1_item_sets_core(&mut self, grammar: &Grammar, start_symbol: &Element) {
    let mut item_sets = Vec::<HashSet<LR1Item>>::new();
    let initial_item = LR1Item {
      head: start_symbol.clone(),
      body: grammar.pro_list.get(start_symbol).unwrap()[0].clone(),
      dot: 0,
      lookahead: Element::Terminal("#".to_string()),
    };

    let mut initial_closure = HashSet::new();
    initial_closure.insert(initial_item);
    let initial_closure = self.closure(grammar, &initial_closure);
    item_sets.push(initial_closure);

    let mut i = 0;
    while i < item_sets.len() {
      let item_set = item_sets[i].clone();

      let terminals = grammar.token_list.iter().map(|s| Element::Terminal(s.clone()));
      let non_terminals = grammar.pro_list.keys().cloned();

      for symbol in terminals.chain(non_terminals) {
        let next_item_set = self.goto(grammar, &item_set, &symbol);

        if !next_item_set.is_empty() && !item_sets.contains(&next_item_set) {
          item_sets.push(next_item_set);
        }
      }

      i += 1;
    }

    self.lr1_sets = item_sets;
  }

  // NOTE: 时间复杂度太大
  // pub fn compute_lr1_item_sets(&mut self, grammar: &Grammar, START_SYMBOL: &Element) {
  //   let mut item_sets = Vec::<HashSet<LR1Item>>::new();
  //
  //   let mut initial_set = HashSet::<LR1Item>::new();
  //   let initial_item = LR1Item {
  //     head: START_SYMBOL.clone(),
  //     body: grammar.pro_list.get(START_SYMBOL).unwrap()[0].clone(),
  //     dot: 0,
  //     lookahead: Element::Terminal("#".to_string()),
  //   };
  //   initial_set.insert(initial_item);
  //   item_sets.push(self.closure(grammar, &initial_set));
  //
  //   let mut changed = true;
  //   while changed {
  //     changed = false;
  //
  //     let mut new_item_set = Vec::<HashSet<LR1Item>>::new();
  //     // let mut new_item_set = Vec::<HashSet<LR1Item>>::new();
  //     for item_set in &item_sets {
  //       for item in item_set {
  //         let dot_position = item.dot;
  //         if dot_position < item.body.len() {
  //           let next_symbol = &item.body[item.dot];
  //           let goto_set = self.goto(grammar, item_set, next_symbol);
  //           if !item_sets.contains(&goto_set) && !new_item_set.contains(&goto_set) {
  //             new_item_set.push(goto_set);
  //             changed = true;
  //           }
  //         }
  //       }
  //     }
  //     item_sets.extend(new_item_set.into_iter());
  //   }
  //   self.lr1_sets = item_sets;
  // }

  fn closure(&self, grammar: &Grammar, item_set: &HashSet<LR1Item>) -> HashSet<LR1Item> {
    let mut closure_set = item_set.clone();

    let mut changed = true;
    while changed {
      changed = false;

      for item in closure_set.clone() {
        let dot_position = item.dot;
        if dot_position < item.body.len() {
          let next_symbol = &item.body[dot_position];

          match next_symbol {
            Element::NotTerminal(_) => {
              let lookahead_symbols = grammar.first_symbols(&item.body[(dot_position + 1)..], &item.lookahead);
              let productions = grammar.pro_list.get(next_symbol).unwrap();

              for production in productions {
                for lookahead_symbol in &lookahead_symbols {
                  let new_item = LR1Item {
                    head: next_symbol.clone(),
                    body: production.clone(),
                    dot: 0,
                    lookahead: lookahead_symbol.clone(),
                  };

                  if !closure_set.contains(&new_item) {
                    closure_set.insert(new_item);
                    changed = true;
                  }
                }
              }
            }
            Element::Terminal(_) => {}
          }
        }
      }
    }
    closure_set
  }

  fn goto(&self, grammar: &Grammar, item_set: &HashSet<LR1Item>, symbol: &Element) -> HashSet<LR1Item> {
    let mut goto_set = HashSet::new();

    for item in item_set {
      if item.dot < item.body.len() && &item.body[item.dot] == symbol {
        let mut new_item = item.clone();
        new_item.dot += 1;
        goto_set.insert(new_item);
      }
    }

    self.closure(grammar, &goto_set)
  }
}


fn get_exception_symbols(action_table: &ActionTable, state: &State) -> Vec<Element> {
  let mut exception_symbols = Vec::new();

  for action in action_table {
    if action.0.0 == state.clone() && action.0.1 != Element::Terminal("#".to_string()) {
      exception_symbols.push(action.0.1.clone());
    }
  }

  exception_symbols
}

// 迭代错误恢复
fn iterative_error_recovery(
  action_table: &ActionTable,
  goto_table: &GotoTable,
  state_stack: &mut Vec<State>,
  node_stack: &mut Vec<TreeNode>,
  input_buff: &mut Vec<Element>,
  input_pos: &mut usize,
) {
  // while循环，当action_table中没有state和symbol对应的action时，继续迭代
  while action_table.get(&(state_stack.last().unwrap().clone(), input_buff[*input_pos].clone())).is_none() {
    // 当前最小代价
    let mut min_cost = std::isize::MAX;
    let mut best_choice = "";

    // 尝试插入操作
    let exception_symbols = get_exception_symbols(action_table, state_stack.last().unwrap());
    for exception_symbol in &exception_symbols {
      let cost = get_insert_cost(input_buff, input_pos, &input_buff[*input_pos].clone());
      if cost < min_cost {
        min_cost = cost;
        best_choice = "insert";
      }
    }

    // 尝试删除操作
    let cost = get_delete_cost(input_buff, input_pos, &input_buff[*input_pos].clone());
    if cost < min_cost {
      min_cost = cost;
      best_choice = "delete";
    }

    // 尝试替换操作
    for exception_symbol in &exception_symbols {
      // exception_symbol和symbol不相等
      if exception_symbol != &input_buff[*input_pos].clone() {
        let cost = get_replace_cost(input_buff, input_pos, &input_buff[*input_pos].clone());
        if cost < min_cost {
          min_cost = cost;
          best_choice = "replace";
        }
      }
    }

    match best_choice {
      "insert" => {
        // 插入操作：将插入的符号压入符号栈，根据当前状态和插入符号找到转移后的状态，将其压入状态栈。
        // let state_prime = state_stack.last().unwrap();
        // let goto_state = action_table.get(&(state_prime.clone(), exception_symbols.last().unwrap().clone())).unwrap();
        //
        // state_stack.push(*goto_state);
        // node_stack.push(TreeNode { element: exception_symbols.last().unwrap().clone(), children: None });
        input_buff.insert(*input_pos, exception_symbols.last().unwrap().clone());
        // *input_pos += 1;
        println!("insert {:?}", input_buff);
      }
      "delete" => {
        println!("delete {:?}", input_buff[*input_pos].clone());
        state_stack.pop();
        node_stack.pop();
      }
      "replace" => {
        println!("replace {:?} with {:?}", input_buff[*input_pos].clone(), exception_symbols[0]);
        // let state_prime = state_stack.last().unwrap();
        // let goto_state = goto_table.get(&(state_prime.clone(), exception_symbols.last().unwrap().clone())).unwrap();
        // state_stack.pop();
        // node_stack.pop();
        // state_stack.push(*goto_state);
        // node_stack.push(TreeNode { element: exception_symbols.last().unwrap().clone(), children: None });
        state_stack.pop();
        node_stack.pop();
        input_buff.insert(*input_pos, exception_symbols.last().unwrap().clone());
      }
      _ => {
        println!("????");
      }
    }
  }
}


fn get_delete_cost(input_buff: &Vec<Element>, input_pos: &usize, symbol: &Element) -> isize {
  let mut cost: isize = 0;
  // 在input_pos之前，如果有symbol，则cost-1，在input_pos之后，如果有symbol，则cost不变，其他情况cost+1
  if input_buff[*input_pos - 1] == *symbol {
    cost -= 1;
  } else if input_buff[input_pos + 1] == *symbol {
    cost += 0;
  } else {
    cost += 1;
  }
  cost
}

fn get_insert_cost(input_buff: &Vec<Element>, input_pos: &usize, symbol: &Element) -> isize {
  let mut cost: isize = 2;
  // // 当前symbol为';'，cost-1，否则cost+1
  // if symbol == &Element::Terminal(";".to_string()) {
  //   cost -= 1;
  // } else {
  //   cost -= 1;
  // }

  cost
}

fn get_replace_cost(input_buff: &Vec<Element>, input_pos: &usize, symbol: &Element) -> isize {
  let mut cost: isize = 2;
  // 当前symbol为';'，cost-1，否则cost+1
  // if symbol == &Element::Terminal(";".to_string()) {
  //   cost -= 1;
  // } else {
  //   cost -= 1;
  // }

  cost
}