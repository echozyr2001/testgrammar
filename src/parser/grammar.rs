use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::Read;
use crate::parser::{EMPTY_SYMBOL, START_SYMBOL};
use crate::parser::types::{Token, Item, PBody, PHead};

#[derive(Debug)]
pub struct Grammar {
  file_buff: String,
  pub(crate) token_list: Vec<String>,
  pub(crate) pro_list: BTreeMap<PHead, PBody>,
  pub(crate) first_sets: BTreeMap<Token, BTreeSet<Token>>,
  pub(crate) start_symbol: Token,
}

impl Grammar {
  pub fn new() -> Self {
    Self {
      token_list: Vec::<String>::new(),
      pro_list: BTreeMap::<PHead, PBody>::new(),
      first_sets: BTreeMap::<Token, BTreeSet<Token>>::new(),
      file_buff: String::new(),
      start_symbol: START_SYMBOL.clone(),
    }
    // Self::default()
  }

  fn file_load(&mut self, file_path: &str) {
    let mut file = File::open(file_path).expect("File open Err!!{}");
    file.read_to_string(&mut self.file_buff).expect("File read Err!!");
  }

  pub fn grammar_load(&mut self, grammar_path: &str) {
    self.file_load(grammar_path);
    let lines = self.file_buff.lines();
    for line in lines {
      if line.starts_with('{') { continue; }
      if line.starts_with('}') { continue; }

      if line.starts_with("%token") {
        let tokens = line.split_whitespace().skip(1);
        for token in tokens {
          self.token_list.push(token.to_string());
        }
        continue;
      }
      {
        let mut tmp = line.split(':'); // 只有可能是两部分
        // let p_head = PHead::NotTerminal(tmp.next().unwrap().to_string());
        let p_head = Token::new_not_terminal(tmp.next().unwrap().to_string(), None);
        let mut p_body = PBody::new();
        let items = tmp.next().unwrap().split("#|#"); // 拆分右部
        for item in items {
          let elements = item.split_whitespace();
          let mut item = Item::new();
          for element in elements {
            item.push(
              if self.token_list.contains(&element.to_string()) {
                // Element::Terminal(element.to_string())
                Token::new_terminal(element.to_string(), None)
              } else {
                // Element::NotTerminal(element.to_string())
                Token::new_not_terminal(element.to_string(), None)
              }
            );
          }
          p_body.push(item);
        }
        self.pro_list.insert(p_head, p_body);
      }
    }

    self.calculate_first_sets();
  }

  fn first(&mut self, symbol: &Token) -> BTreeSet<Token> {
    if let Some(first_set) = self.first_sets.get(symbol) {
      return first_set.clone();
    }

    let mut result = BTreeSet::new();
    match symbol {
      token if token.is_terminal() => {
        result.insert(symbol.clone());
      }
      token if token.is_not_terminal() => {
        if let Some(productions) = self.pro_list.get(symbol) {
          let productions: Vec<Vec<_>> = productions.to_vec();
          for production in productions {
            let first_symbol = &production[0];
            match first_symbol {
              token if token.is_terminal() => {
                result.insert(first_symbol.clone());
              }
              token if token.is_not_terminal() => {
                let mut first_set = self.first(first_symbol);
                let mut i = 1;
                let empty_symbol = EMPTY_SYMBOL.clone();
                while i < production.len() && first_set.contains(&empty_symbol) {
                  first_set.remove(&empty_symbol);
                  result.extend(first_set);
                  let next_symbol = &production[i];
                  first_set = self.first(next_symbol);
                  i += 1;
                }
                result.extend(first_set);
              }
              _ => { unreachable!() }
            }
          }
        }
      }
      _ => { unreachable!() }
    }

    self.first_sets.insert(symbol.clone(), result.clone());
    result
  }

  fn calculate_first_sets(&mut self) {
    let non_terminals: Vec<_> = self.pro_list.keys().cloned().collect();

    for non_terminal in non_terminals {
      self.first(&non_terminal);
    }
  }

  pub(crate) fn first_symbols(&self, symbols: &[Token], fallback: &Token) -> BTreeSet<Token> {
    let mut result = BTreeSet::new();
    let mut epsilon = true;

    for symbol in symbols {
      epsilon = false;

      match symbol {
        token if token.is_terminal() => {
          result.insert(symbol.clone());
          break;
        }
        token if token.is_not_terminal() => {
          let first_set = self.first_sets.get(symbol).unwrap();
          let empty_symbol = EMPTY_SYMBOL.clone();

          if first_set.contains(&empty_symbol) {
            epsilon = true;
            result.extend(first_set.clone().into_iter().filter(|x| *x != empty_symbol));
          } else {
            result.extend(first_set.clone().into_iter());
            break;
          }
        }
        _ => { unreachable!() }
      }
    }

    if epsilon {
      result.insert(fallback.clone());
    }

    result
  }
}