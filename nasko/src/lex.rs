use crate::semantics::{NaskoArithmetic, NaskoType, NaskoKeyword};
use logos::Logos;

#[derive(Default)]
pub struct NaskoExtras {
  pub line_caps: Vec<usize>,
  pub line_breaks: usize,
  pub spaces: usize
}

impl NaskoExtras {
  #[allow(dead_code)]
  pub fn line_heads(&self) -> Vec<usize> {
    let mut heads: Vec<usize> = vec![0];

    let mut i = 0;
    for _ in self.line_caps.iter() {
      if i == 0 {
        i += 1;
        continue
      } else {
        heads.push(self.line_caps[i] + 1);
      }
      i += 1;
    }

    heads
  }
}

#[derive(Logos, Debug, PartialEq)]
#[logos(extras = NaskoExtras)]
pub enum NaskoToken {
  #[token("\n", |lex| {
    lex.extras.line_breaks += 1;
    lex.extras.line_caps.push(lex.span().start);

    logos::Skip
  })]
  #[regex(r"[ \t\f]", |lex| {
    lex.extras.spaces += 1;

    logos::Skip
  })]
  #[error]
  Error,

  #[regex(r#""([^"\t\n])*""#, |lex| lex.slice().to_string())]
  LiteralString(String),

  #[regex("0[xX]([0-9a-fA-F])+", 
    |lex| i64::from_str_radix(&lex.slice()[2..], 16))]
  LiteralHex(i64),

  #[regex("-?[0-9]+(\\.[0-9]+)?", |lex| lex.slice().parse())]
  LiteralNumber(f64),

  #[regex("true|false", |lex| lex.slice() == "true")]
  LiteralBoolean(bool),

  #[regex("//.*", |lex| lex.slice().replace("//", "").trim().to_string())]
  Comment(String),

  #[regex("func|struct|enum|import|if|return|while|let|as", |lex| NaskoKeyword::parse(lex.slice()))]
  Keyword(NaskoKeyword),

  #[regex("\\+|-|/|\\*|%|\\*\\*", |lex| NaskoArithmetic::parse(lex.slice()))]
  ArithmeticOperator(NaskoArithmetic),

  #[regex("string|int|boolean", |lex| NaskoType::parse(lex.slice()))]
  TypeAnnotation(NaskoType),

  #[regex("[a-zA-Z_]([a-zA-Z0-9_]+)?", |lex| lex.slice().to_string())]
  Ident(String),

  #[token("{")]
  BlockOpen,

  #[token("}")]
  BlockClose,

  #[token("[")]
  SubscriptOpen,

  #[token("]")]
  SubscriptClose,

  #[token("(")]
  ParenOpen,

  #[token(")")]
  ParenClose,

  #[token(",")]
  Comma,

  #[token(".")]
  Dot,

  #[token(";")]
  Semicolon,

  #[token(":")]
  Colon,

  #[token("=")]
  Eq,
}
