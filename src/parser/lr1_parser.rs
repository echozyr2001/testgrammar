#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(clippy::clone_on_copy)]
use crate::parser::types::Element;
use crate::parser::{Grammar, ACTION_TABLE, DATA_PATH, GOTO_TABLE, LR1_SETS};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::fs::{create_dir_all, metadata, File};
use std::hash::Hash;

type State = usize;
type GotoTable = HashMap<(State, Element), State>;
type ActionTable = HashMap<(State, Element), Action>;
type LR1Sets = Vec<HashSet<LR1Item>>;
type ErrorList = Vec<ParserError>;

#[derive(Debug, Default)]
pub struct LR1Parser {
    tokens: Vec<Element>,
    error_list: Vec<ParserError>,
    status: Status,
    part_table: PartitionTable,
    pos: usize,
    pub lr1_sets: LR1Sets,
    pub action_table: ActionTable,
    pub goto_table: GotoTable,
}

impl LR1Parser {
    pub fn new() -> Self {
        Self::default()
    }
    fn get_last_token(&self) -> &Element {
        &self.tokens[if self.pos == 0 { 0 } else { self.pos - 1 }]
    }
    fn get_current_token(&self) -> &Element {
        &self.tokens[if self.pos >= self.tokens.len() {
            self.tokens.len()
        } else {
            self.pos
        }]
    }
}

impl LR1Parser {
    pub fn construct_tree(mut self, input: &[Element]) -> Self {
        self.tokens = input.to_owned();
        self.tokens.push(Element::Terminal("#".to_string()));

        loop {
            if self.pos >= self.tokens.len() {
                break;
            }
            let state = self.status.state_stack.last().unwrap().clone();
            let symbol = self.tokens[self.pos].clone();

            let action = self
                .action_table
                .get(&(state.clone(), symbol.clone()))
                .cloned();

            match action {
                Some(Action::Shift(state)) => {
                    self.status.state_stack.push(state);
                    self.status.node_stack.push(TreeNode {
                        element: symbol.clone(),
                        children: None,
                    });
                    self.step_forward();
                    self.try_partition();
                }
                Some(Action::Reduce(prod_head, prod_body)) => {
                    let mut children: Vec<TreeNode> = Vec::new();
                    for _ in 0..prod_body.len() {
                        self.status.state_stack.pop();
                        children.push(self.status.node_stack.pop().unwrap());
                    }
                    children.reverse();

                    let state = self.status.state_stack.last().unwrap().clone();
                    let state = self
                        .goto_table
                        .get(&(state, prod_head.clone()))
                        .unwrap()
                        .clone();

                    self.status.state_stack.push(state);
                    self.status.node_stack.push(TreeNode {
                        element: prod_head,
                        children: Some(children),
                    });
                }
                Some(Action::Accept) => {
                    break;
                }
                None => {
                    if let Some(Action::Shift(t)) = self
                        .action_table
                        .get(&(state.clone(), Element::Terminal("ε".to_string())))
                        .cloned()
                    {
                        self.status.state_stack.push(t);
                        self.status.node_stack.push(TreeNode {
                            element: Element::Terminal("ε".to_string()),
                            children: None,
                        });
                    } else {
                        self.err_handle();
                    }
                }
            }
        }
        self
    }

    fn err_handle(&mut self) {
        // error_list.(format!("Unexpected symbol '{:?}' at position {}", symbol, input_pos));
        self.error_list.push(ParserError {
            error_type: ErrorType::Unknown(format!(
                "Unexpected symbol '{:?}' at position {}",
                self.tokens[self.pos], self.pos
            )),
            error_pos: self.tokens[self.pos].get_pos(),
        });

        // 错误处理：跳过输入直到找到一个可以接受的符号
        let mut found_acceptable_symbol = false;
        while !found_acceptable_symbol {
            self.step_forward();

            if self.pos >= self.tokens.len() {
                break;
            }

            let symbol = Delimiter::from(self.get_current_token().clone());
            if symbol.is_limiter() {
                self.fallback();
                // self.step_forward();
                break;
            }

            let next_symbol = self.tokens[self.pos].clone();
            if self
                .action_table
                .get(&(
                    self.status.state_stack.last().unwrap().clone(),
                    next_symbol.clone(),
                ))
                .is_some()
            {
                found_acceptable_symbol = true;
            }
        }
    }
    fn step_forward(&mut self) {
        self.pos += 1;
        // let last_token = self.get_last_token().clone();
        // if last_token.is_left_bracket() {
        //     self.status_stack.push((last_token, self.status.clone()));
        // } else if last_token.is_right_bracket() {
        //     let mut err = false;
        //     loop {
        //         if let Some((token, status)) = self.status_stack.pop() {
        //             if token.is_paired(&last_token) {
        //                 if err {
        //                     self.status = status;
        //                 }
        //                 break;
        //             } else {
        //                 err = true;
        //                 self.error_list.push(ParserError {
        //                     error_type: ErrorType::Missing(token.clone().unwarp().0),
        //                     error_pos: (token.get_pos()),
        //                 })
        //             }
        //         } else {
        //             self.status = Status::default();
        //             break;
        //         }
        //     }
        // }
    }
    fn fallback(&mut self) {
        self.status = self.part_table.get_last_status()
    }
    fn try_partition(&mut self) {
        let symbol = Delimiter::from(self.get_last_token().clone());
        if symbol.is_delimiter() {
            self.part_table.update(Partition {
                partition_head: symbol,
                status: self.status.clone(),
            })
        }
    }
}

#[derive(Debug, Default)]
struct PartitionTable {
    table: Vec<Partition>,
}
impl PartitionTable {
    fn get_last_token(&self) -> Delimiter {
        if let Some(v) = self.table.last() {
            v.partition_head
        } else {
            Delimiter::Default
        }
    }
    fn get_last_status(&self) -> Status {
        if let Some(v) = self.table.last() {
            v.status.clone()
        } else {
            Status::default()
        }
    }
    fn update(&mut self, partition: Partition) {
        let token = partition.partition_head;
        match token {
            Delimiter::Semi
            | Delimiter::OpenParen
            | Delimiter::OpenBrace
            | Delimiter::OpenBracket => {
                if self.get_last_token() == Delimiter::Semi {
                    self.table.pop();
                    self.table.push(partition);
                } else {
                    self.table.push(partition);
                }
            }
            Delimiter::CloseParen | Delimiter::CloseBrace | Delimiter::CloseBracket => {
                if self.get_last_token() == Delimiter::Semi {
                    self.table.pop();
                    self.table.pop();
                } else {
                    self.table.pop();
                }
            }
            Delimiter::Default => (),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Status {
    state_stack: Vec<State>,
    node_stack: Vec<TreeNode>,
}
impl Default for Status {
    fn default() -> Self {
        let state_stack = vec![0];
        let node_stack = vec![TreeNode {
            element: Element::NotTerminal("#".to_string()),
            children: None,
        }];
        Self {
            state_stack,
            node_stack,
        }
    }
}
#[derive(Debug)]
pub struct Partition {
    partition_head: Delimiter,
    status: Status,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delimiter {
    /// ";"
    Semi,
    /// "("
    OpenParen,
    /// ")"
    CloseParen,
    /// "{"
    OpenBrace,
    /// "}"
    CloseBrace,
    /// "["
    OpenBracket,
    /// "]"
    CloseBracket,
    Default,
}
impl Delimiter {
    fn is_delimiter(&self) -> bool {
        self != &Self::Default
    }
    fn is_limiter(&self) -> bool {
        matches!(
            self,
            Delimiter::Semi
                | Delimiter::CloseParen
                | Delimiter::CloseBrace
                | Delimiter::CloseBracket
        )
    }
}
impl From<Element> for Delimiter {
    fn from(v: Element) -> Self {
        match v.unwarp().0.as_str() {
            "';'" => Delimiter::Semi,
            "'('" => Delimiter::OpenParen,
            "')'" => Delimiter::CloseParen,
            "'{'" => Delimiter::OpenBrace,
            "'}'" => Delimiter::CloseBrace,
            "'['" => Delimiter::OpenBracket,
            "']'" => Delimiter::CloseBracket,
            _ => Delimiter::Default,
        }
    }
}
impl Default for Partition {
    fn default() -> Self {
        Self {
            partition_head: Delimiter::Default,
            status: Default::default(),
        }
    }
}

#[derive(Debug)]
pub struct ParserError {
    pub error_type: ErrorType,
    pub error_pos: usize,
}

#[derive(Debug)]
pub enum ErrorType {
    // 缺失错误
    Missing(String),
    // 多余错误
    Extra(String),
    // 未知错误
    Unknown(String),
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

impl Display for LR1Parser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.error_list.is_empty() {
            write!(f, "{}", self.status.node_stack.last().unwrap())
        } else {
            let mut buffer = String::new();
            for i in self.error_list.iter() {
                buffer.push_str(&format!("{:?}", i));
                buffer.push('\n');
            }
            write!(f, "{}", buffer)
        }
    }
}
#[allow(clippy::print_in_format_impl)]
impl Display for TreeNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
        print_tree(self, 0);
        writeln!(f)
    }
}

impl LR1Parser {
    pub fn construct_parsing_table(&mut self, grammar: &Grammar) {
        if file_exists(ACTION_TABLE) && file_exists(GOTO_TABLE) {
            let action_file = File::open(ACTION_TABLE).expect("Unable to open action table file");
            let goto_file = File::open(GOTO_TABLE).expect("Unable to open goto table file");
            self.action_table = bincode::deserialize_from(action_file).unwrap();
            self.goto_table = bincode::deserialize_from(goto_file).unwrap();
        } else {
            self.construct_parsing_table_core(grammar);

            create_dir_all(DATA_PATH).expect("Unable to create action table file");
            let mut action_file =
                File::create(ACTION_TABLE).expect("Unable to create action table file");
            let mut goto_file = File::create(GOTO_TABLE).expect("Unable to create goto table file");
            bincode::serialize_into(&mut action_file, &self.action_table)
                .expect("Unable to serialize action table");
            bincode::serialize_into(&mut goto_file, &self.goto_table)
                .expect("Unable to serialize goto table");
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
                        let goto_set =
                            self.goto(grammar, item_set, &Element::Terminal(a.to_string()));
                        let goto_state = self.lr1_sets.iter().position(|x| *x == goto_set).unwrap();
                        self.action_table.insert(
                            (state, Element::Terminal(a.to_string())),
                            Action::Shift(goto_state),
                        );
                    }
                    Some(Element::NotTerminal(a)) => {
                        let goto_set =
                            self.goto(grammar, item_set, &Element::NotTerminal(a.to_string()));
                        let goto_state = self.lr1_sets.iter().position(|x| *x == goto_set).unwrap();
                        self.goto_table
                            .insert((state, Element::NotTerminal(a.to_string())), goto_state);
                    }
                    None => {
                        if item.head == grammar.start_symbol
                            && item.lookahead == Element::Terminal("#".to_string())
                        {
                            self.action_table.insert(
                                (state, Element::Terminal("#".to_string())),
                                Action::Accept,
                            );
                        } else {
                            let prod_index = grammar
                                .pro_list
                                .get(&item.head)
                                .unwrap()
                                .iter()
                                .position(|x| *x == item.body)
                                .unwrap();
                            self.action_table.insert(
                                (state, item.lookahead.clone()),
                                Action::Reduce(
                                    item.head.clone(),
                                    grammar.pro_list.get(&item.head).unwrap()[prod_index].clone(),
                                ),
                            );
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
            bincode::serialize_into(&mut lr1_file, &self.lr1_sets)
                .expect("Unable to serialize action table");
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

            let terminals = grammar
                .token_list
                .iter()
                .map(|s| Element::Terminal(s.clone()));
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
                            let lookahead_symbols = grammar
                                .first_symbols(&item.body[(dot_position + 1)..], &item.lookahead);
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

    fn goto(
        &self,
        grammar: &Grammar,
        item_set: &HashSet<LR1Item>,
        symbol: &Element,
    ) -> HashSet<LR1Item> {
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
        if action.0 .0 == state.clone() && action.0 .1 != Element::Terminal("#".to_string()) {
            exception_symbols.push(action.0 .1.clone());
        }
    }

    exception_symbols
}