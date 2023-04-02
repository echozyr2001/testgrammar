mod parser;

use crate::parser::Element;

fn main() {
    let path = "./g2.txt";
    let mut grammar = parser::Grammar::new();
    grammar.grammar_load(path);

    let mut lr1 = parser::LR1Parser::new();
    lr1.compute_lr1_item_sets(&grammar);
    lr1.construct_parsing_table(&grammar);

    #[rustfmt::skip]
    let input:Vec<Element> = vec![
        // "'int'", "Ident", "'('","')'"  ,"'}'",

        "'int'", "Ident", "'('","')'"  ,"'{'",
        "'const'", "'int'", "Ident", "'='", "'{'","errIntConst", "';'", 
        "'const'", "'int'", "Ident", "'='", "errIntConst", "';'",
        "'}'",
        //         "'int'", "Ident", "'('","')'"  ,"'{'","'}'",
        // "'int'", "Ident", "'('","')'"  ,"'{'","'}'",
        "'const'", "'int'", "Ident", "'='", "'{'","IntConst","'}'", "';'", 
        // "'const'", "'int'", "Ident", "'='", "errIntConst", "';'",
        "'const'", "'int'", "Ident", "'='", "IntConst", "';'", 
        "'const'", "'int'", "Ident", "'='", "IntConst", "';'", 
        // "'const'", "'int'", "Ident", "'='", "errIntConst","';'",
        // "'const'", "'int'", "Ident", "IntConst", "';'", 
        "'const'", "'int'", "Ident", "'='", "errIntConst", "';'",
        "'const'", "'float'", "Ident", "'='", "FloatConst", "';'",
    ]
    .into_iter()
    .map(|e| Element::Terminal(e.to_string()))
    .collect();

    let tmp = lr1.construct_tree(&input);
    print!("{}", tmp);
}

// 递归输出语法树, 使用depth来控制缩进
