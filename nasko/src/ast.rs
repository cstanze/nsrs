#![allow(dead_code)]
use std::fmt;
use crate::semantics::*;
use nasko_proc_macro::GenericASTNode;

#[derive(Clone)]
pub enum ExtraNodeData {
  String(String),
  Number(f64),
  Boolean(bool),
  Vec(Vec<ExtraNodeData>),
  BoxedNode(Box<dyn ASTNode>),
  None
}

impl fmt::Debug for ExtraNodeData {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ExtraNodeData::String(s) => write!(f, "{}", s),
      ExtraNodeData::Number(n) => write!(f, "{}", n),
      ExtraNodeData::Boolean(b) => write!(f, "{}", b),
      ExtraNodeData::Vec(v) => write!(f, "{:?}", v),
      ExtraNodeData::BoxedNode(b) => write!(f, "{:?}", b),
      ExtraNodeData::None => write!(f, "None"),
    }
  }
}

impl Default for ExtraNodeData {
  fn default() -> Self {
    ExtraNodeData::None
  }
}

pub trait ASTClone {
  fn clone_box(&self) -> Box<dyn ASTNode>;
}

impl<T> ASTClone for T
where
  T: 'static + ASTNode + Clone
{
  fn clone_box(&self) -> Box<dyn ASTNode> {
    Box::new(self.clone())
  }
}

impl Clone for Box<dyn ASTNode> {
  fn clone(&self) -> Box<dyn ASTNode> {
    self.clone_box()
  }
}

pub trait ASTNode: ASTClone {
  /// Formatted Node
  fn debug_fmt(&self) -> String;
  /// Node Type
  fn node_type(&self) -> &str;
  /// Gets all children as &Vec
  fn get_leaves(&self) -> &Vec<Box<dyn ASTNode>>;
  /// Add a leaf to the tree branch
  fn push_leaf(&mut self, data: Box<dyn ASTNode>);
  /// Get a mutable reference to a leaf
  fn get_leaf_mut(&mut self, index: usize) -> Option<&mut Box<dyn ASTNode>>;
}

impl fmt::Debug for dyn ASTNode {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.debug_fmt())
  }
}

#[derive(GenericASTNode, Default, Debug, Clone)]
pub struct SourceNode {
  #[node_type(|| "Source")]

  pub ta: NaskoType,

  #[children]
  pub children: Vec<Box<dyn ASTNode>>,
}

#[derive(GenericASTNode, Debug, Clone)]
pub struct EmptyNode {
  #[node_type(|| "Empty")]

  pub ta: NaskoType,

  #[children]
  pub children: Vec<Box<dyn ASTNode>>,
}

#[derive(GenericASTNode, Debug, Clone)]
pub struct ValueNode {
  #[node_type(|| &self.ntype)]

  pub ta: NaskoType,
  pub ntype: String,
  pub value: ExtraNodeData,

  #[children]
  pub children: Vec<Box<dyn ASTNode>>,
}

#[derive(GenericASTNode, Debug, Clone)]
pub struct FunctionDeclNode {
  #[node_type(|| "FunctionDecl")]

  pub ta: NaskoType,
  pub value: ExtraNodeData,
  pub params: Vec<Box<ValueNode>>,

  #[children]
  pub children: Vec<Box<dyn ASTNode>>
}

#[derive(GenericASTNode, Debug, Clone)]
pub struct NameNode {
  #[node_type(|| "Ident")]

  pub ta: NaskoType,
  pub name: String,

  #[children]
  pub children: Vec<Box<dyn ASTNode>>,
}

#[derive(GenericASTNode, Debug, Clone)]
pub struct BlockNode {
  #[node_type(|| "Block")]

  pub ta: NaskoType,

  #[children]
  pub children: Vec<Box<dyn ASTNode>>,
}

#[derive(GenericASTNode, Debug, Clone)]
pub struct StatementNode {
  #[node_type(|| &self.ntype)]

  pub ta: NaskoType,
  pub ntype: String,

  #[children]
  pub children: Vec<Box<dyn ASTNode>>,
}

#[derive(GenericASTNode, Debug, Clone)]
pub struct ExpressionNode {
  #[node_type(|| &self.ntype)]
  
  pub ta: NaskoType,
  pub ntype: String,

  #[children]
  pub children: Vec<Box<dyn ASTNode>>,
}

#[derive(GenericASTNode, Debug, Clone)]
pub struct BinaryExpression {
  #[node_type(|| "BinaryExpression")]
  
  pub ta: NaskoType,
  pub rhs: Option<Box<dyn ASTNode>>,
  pub lhs: Option<Box<dyn ASTNode>>,
  pub expression: NaskoArithmetic,

  #[children]
  pub children: Vec<Box<dyn ASTNode>>
}
