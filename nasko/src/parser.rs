#![allow(dead_code)]

#[allow(unused_imports)]
use crate::{
  ast::*,
  macros::compile_error,
  lex::NaskoToken,
  semantics::{
    NaskoArithmetic,
    NaskoKeyword,
    NaskoType
  }
};
use logos::Lexer;

#[derive(Debug, Default)]
pub struct ParserState {
  pub in_function_decl: bool,
  pub in_struct_decl: bool,
  
  pub in_arg: bool,
  pub in_return: bool,
  
  pub in_parens: bool,
  pub in_colon: bool,
  pub in_comma: bool,
  pub in_block: bool,
  pub in_expression: bool,
  
  pub in_subscript: bool,
  pub subscript_has_input: bool,
  pub subscript_input: SubscriptInput,
  
  pub closed_semi: bool,

  pub has_unknown_ident: bool,
}

#[derive(Debug)]
pub enum SubscriptInput {
  None,
  Number(f64),
  String(String),
}

impl Default for SubscriptInput {
  fn default() -> Self {
    Self::None
  }
}

/// Pushes a new leaf into the vec of a ValueNode
///
/// Replaces existing value if it is not vec data
fn vec_upsert(tree: &mut ValueNode, data: Box<dyn ASTNode>) {
  match &mut tree.value {
    ExtraNodeData::Vec(v) => {
      v.push(ExtraNodeData::BoxedNode(data))
    },
    _ => tree.value = ExtraNodeData::Vec(vec![ExtraNodeData::BoxedNode(data)])
  }
}


/// Gets a mutable reference to a vec of a ValueNode
///
/// The vec element type is always ExtraNodeData
///
/// Returns `None` if the value is not `ExtraNodeData::Vec`
fn vec_get_mut(tree: &mut ValueNode) -> Option<&mut Vec<ExtraNodeData>> {
  match &mut tree.value {
    ExtraNodeData::Vec(v) => Some(v),
    _ => None
  }
}


pub fn parse(lex: &mut Lexer<NaskoToken>) -> Box<dyn ASTNode> {
  let mut tree = SourceNode::default();
  let mut state = ParserState::default();

  loop {
    // println!("{:#?}", tree);

    match lex.next() {
      None => break,
      Some(t) => {
        if t != NaskoToken::Semicolon {
          state.closed_semi = false;
        }
        match t {
          NaskoToken::ParenOpen => state.in_parens = true,
          NaskoToken::ParenClose => {
            state.in_arg = false;
            state.in_parens = false;
            state.has_unknown_ident = false;
          },
          NaskoToken::BlockOpen => state.in_block = true,
          NaskoToken::BlockClose => {
            state.in_struct_decl = false;
            state.in_function_decl = false;
            state.in_block = false
          },
          NaskoToken::Colon => state.in_colon = true,
          NaskoToken::Comma => {
            if state.in_arg {
              state.in_arg = false;
            }
            state.in_comma = true
          },
          NaskoToken::SubscriptOpen => state.in_subscript = true,
          NaskoToken::SubscriptClose => {
            state.in_subscript = false;
          },
          NaskoToken::Semicolon => {
            state.closed_semi = true;
          },

          NaskoToken::Ident(ident) => {
            if state.in_function_decl {
              match state.in_parens {
                true => {
                  let mut fd = pop_leaf!(tree.children, Box<FunctionDeclNode>);
                  fd.params.push(Box::new(ValueNode {
                    ntype: "FunctionArgumentDecl".to_string(),
                    value: ExtraNodeData::String(ident),
                    children: vec![],
                    ta: NaskoType::Unknown
                  }));
                  tree.push_leaf(fd);
                  state.in_arg = true;
                },
                false => {
                  if !state.in_block {
                    let mut fd = pop_leaf!(tree.children, Box<FunctionDeclNode>);
                    fd.value = ExtraNodeData::String(ident);
                    tree.push_leaf(fd);
                  } else {
                    let mut fd = pop_leaf!(tree.children, Box<FunctionDeclNode>);
                    fd.push_leaf(Box::new(ValueNode {
                      ta: NaskoType::Unknown,
                      ntype: "UnknownIdent".to_string(),
                      value: ExtraNodeData::String(ident),
                      children: vec![]
                    }));
                    state.has_unknown_ident = true;
                    tree.push_leaf(fd);
                  }
                }
              }
            }
          },
          NaskoToken::LiteralString(s) => {
            literal!(tree, state, lex, || {
              Box::new(ValueNode {
                ntype: "Constant".to_string(),
                value: ExtraNodeData::String(s),
                children: vec![],
                ta: NaskoType::String
              }) 
            });
          },
          NaskoToken::LiteralBoolean(b) => {
            literal!(tree, state, lex, || {
              Box::new(ValueNode {
                ntype: "Constant".to_string(),
                value: ExtraNodeData::Boolean(b),
                children: vec![],
                ta: NaskoType::Boolean
              }) 
            });
          },
          NaskoToken::LiteralNumber(n) => {
            literal!(tree, state, lex, || {
              Box::new(ValueNode {
                ntype: "Constant".to_string(),
                value: ExtraNodeData::Number(n),
                children: vec![],
                ta: NaskoType::Number
              }) 
            });
          },
          NaskoToken::LiteralHex(n) => {
            literal!(tree, state, lex, || {
              Box::new(ValueNode {
                ntype: "Constant".to_string(),
                value: ExtraNodeData::Number(n as f64),
                children: vec![],
                ta: NaskoType::Number
              }) 
            });
          },
          NaskoToken::ArithmeticOperator(op) => {
            expr_push!(tree, state, lex, |c| {
              tree.push_leaf(Box::new(BinaryExpression {
                rhs: Some(c),
                lhs: None,
                expression: op,
                children: vec![],
                ta: NaskoType::Unknown
              }));
            });

            state.in_expression = true;
          },
          NaskoToken::TypeAnnotation(t) => {
            if t == NaskoType::Unknown {
              continue;
            }
            
            if state.in_function_decl && state.in_colon {
              match state.in_arg {
                true => {
                  let mut fd = pop_leaf!(tree.children, Box<FunctionDeclNode>);
                  let mut ad = pop_leaf!(fd.params, Box<ValueNode>);
                  ad.ta = t;

                  fd.params.push(ad);
                  tree.push_leaf(fd);
                },
                false => {
                  let mut fd = pop_leaf!(tree.children, Box<FunctionDeclNode>);
                  fd.ta = t;
                  tree.push_leaf(fd);
                }
              }
            }
          },
          NaskoToken::Keyword(k) => {
            match k {
              NaskoKeyword::Func => {
                tree.push_leaf(Box::new(ValueNode {
                  children: vec![],
                  ntype: "FunctionDecl".to_string(),
                  value: ExtraNodeData::None,
                  ta: NaskoType::Unknown
                }));
                state.in_function_decl = true;
              },
              NaskoKeyword::Struct => todo!(),
              NaskoKeyword::Enum => todo!(),
              NaskoKeyword::Import => todo!(),
              NaskoKeyword::If => todo!(),
              NaskoKeyword::Return => {
                if !state.in_function_decl {
                  compile_error(format!("Invalid `return` statment outside of function declaration ({:?})", lex.span()))
                }

                let mut func = tree.children.pop().unwrap();
                func.push_leaf(Box::new(StatementNode {
                  children: vec![],
                  ntype: "ReturnStatement".to_string(),
                  ta: NaskoType::Unknown
                }));
                tree.push_leaf(func);
                state.in_return = true;
              },
              NaskoKeyword::While => todo!(),
              NaskoKeyword::Let => todo!(),
              NaskoKeyword::As => compile_error("The Nasko `as` keyword has not been implemented yet".to_string())
            };
          },
          NaskoToken::Error => {
            println!("error at span: {:?}\nwith slice: '{}'", lex.span(), lex.slice());
          },
          _ => {
            println!("token: {:?}", t);
            todo!()
          }
        }
      }
    }
  }
  
  Box::new(tree)
}
