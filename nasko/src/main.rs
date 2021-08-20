#[macro_use]
mod macros;
mod type_check;
mod ast;
mod parser;
mod semantics;
mod lex;

use crate::lex::NaskoToken;
use crate::ast::{SourceNode, ASTNode};
use crate::parser::parse;
use crate::type_check::annotate_types;
use std::fs;
use logos::Logos;


fn main() {
    let code = fs::read_to_string("test.nasko")
        .expect("Failed to read file: test.nasko");
    let mut lex = NaskoToken::lexer(&code);

    let mut ast = *mutate_leaf!(parse(&mut lex), Box<SourceNode>);
    annotate_types(&mut ast);

    println!("ast: {:?}", ast);
}
