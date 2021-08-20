#[derive(Debug, PartialEq, Clone)]
pub enum NaskoArithmetic {
  Add,
  Subtract,
  Divide,
  Multiply,
  Modulo,
  Power,
  None
}

impl NaskoArithmetic {
  pub fn parse(slice: &str) -> NaskoArithmetic {
    match slice {
      "+" => NaskoArithmetic::Add,
      "-" => NaskoArithmetic::Subtract,
      "/" => NaskoArithmetic::Divide,
      "*" => NaskoArithmetic::Multiply,
      "%" => NaskoArithmetic::Modulo,
      "**" => NaskoArithmetic::Power,
      _ => NaskoArithmetic::None
    }
  }
}

#[derive(Debug, PartialEq, Clone)]
pub enum NaskoType {
  String,
  Boolean,
  Number,
  Unknown
}

impl NaskoType {
  pub fn parse(slice: &str) -> NaskoType {
    match slice {
      "string" => NaskoType::String,
      "boolean" => NaskoType::Boolean,
      "int" => NaskoType::Number,
      _ => NaskoType::Unknown
    }
  }
}

impl Default for NaskoType {
  fn default() -> Self {
    NaskoType::Unknown
  }
}

#[derive(Debug, PartialEq, Clone)]
pub enum NaskoKeyword {
  Func,
  Struct,
  Enum,
  Import,
  If,
  Return,
  While,
  Let,
  As
}

impl NaskoKeyword {
  pub fn parse(slice: &str) -> NaskoKeyword {
    match slice {
      "func" => NaskoKeyword::Func,
      "struct" => NaskoKeyword::Struct,
      "enum" => NaskoKeyword::Enum,
      "import" => NaskoKeyword::Import,
      "if" => NaskoKeyword::If,
      "return" => NaskoKeyword::Return,
      "while" => NaskoKeyword::While,
      "let" => NaskoKeyword::Let,
      "as" => NaskoKeyword::As,
      _ => NaskoKeyword::Func
    }
  }
}
