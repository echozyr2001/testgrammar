use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use crate::parser::types::{Element, Item, PBody, PHead};

pub struct Grammar {
  file_buff: String,
  pub(crate) token_list: Vec<String>,
  pub(crate) pro_list: HashMap<PHead, PBody>,
  pub(crate) first_sets: HashMap<Element, HashSet<Element>>,
  pub(crate) start_symbol: Element,
}

impl Grammar {
  pub fn new() -> Self {
    Self {
      token_list: Vec::<String>::new(),
      pro_list: HashMap::<PHead, PBody>::new(),
      first_sets: HashMap::<Element, HashSet<Element>>::new(),
      file_buff: String::new(),
      start_symbol: Element::NotTerminal("CompUnit'".to_string()),
    }
  }

  fn file_load(&mut self, file_path: &str) {
    let mut file = File::open(file_path).expect("File open Err!!{}");
    file.read_to_string(&mut self.file_buff).expect("File read Err!!");
  }

  pub fn grammar_load(&mut self, grammar_path: &str) {
    self.file_load(grammar_path);
    let lines = self.file_buff.lines();
    for line in lines {
      if line.starts_with("{") { continue; }
      if line.starts_with("}") { continue; }

      if line.starts_with("%token") {
        let tokens = line.split_whitespace().skip(1);
        for token in tokens {
          self.token_list.push(token.to_string());
        }
        continue;
      }
      {
        let mut tmp = line.split(":"); // 只有可能是两部分
        let p_head = PHead::NotTerminal(tmp.next().unwrap().to_string());
        let mut p_body = PBody::new();
        let items = tmp.next().unwrap().split("#|#"); // 拆分右部
        for item in items {
          let elements = item.split_whitespace();
          let mut item = Item::new();
          for element in elements {
            item.push(
              if self.token_list.contains(&element.to_string()) {
                Element::Terminal(element.to_string())
              } else {
                Element::NotTerminal(element.to_string())
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

  fn first(&mut self, symbol: &Element) -> HashSet<Element> {
    if let Some(first_set) = self.first_sets.get(symbol) {
      return first_set.clone();
    }

    let mut result = HashSet::new();
    match symbol {
      Element::Terminal(_) => {
        result.insert(symbol.clone());
      }
      Element::NotTerminal(_) => {
        if let Some(productions) = self.pro_list.get(symbol) {
          let productions: Vec<Vec<_>> = productions.iter().map(|x| x.clone()).collect();
          for production in productions {
            let first_symbol = &production[0];
            match first_symbol {
              Element::Terminal(_) => {
                result.insert(first_symbol.clone());
              }
              Element::NotTerminal(_) => {
                let mut first_set = self.first(first_symbol);
                let mut i = 1;
                while i < production.len() && first_set.contains(&Element::Terminal("ε".to_string())) {
                  first_set.remove(&Element::Terminal("ε".to_string()));
                  result.extend(first_set);

                  let next_symbol = &production[i];
                  first_set = self.first(next_symbol);
                  i += 1;
                }
                result.extend(first_set);
              }
            }
          }
        }
      }
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

  pub(crate) fn first_symbols(&self, symbols: &[Element], fallback: &Element) -> HashSet<Element> {
    let mut result = HashSet::new();
    let mut epsilon = true;

    for symbol in symbols {
      epsilon = false;

      match symbol {
        Element::Terminal(_) => {
          result.insert(symbol.clone());
          break;
        }
        Element::NotTerminal(_) => {
          let first_set = self.first_sets.get(symbol).unwrap();

          if first_set.contains(&Element::Terminal("ε".to_string())) {
            epsilon = true;
            result.extend(first_set.clone().into_iter().filter(|x| *x != Element::Terminal("ε".to_string())));
          } else {
            result.extend(first_set.clone().into_iter());
            break;
          }
        }
      }
    }

    if epsilon {
      result.insert(fallback.clone());
    }

    result
  }
}