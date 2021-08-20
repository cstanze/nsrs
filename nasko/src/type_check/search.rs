use crate::ast::*;

pub fn func_with_name(name: &str, tree: Vec<Box<dyn ASTNode>>) -> Option<FunctionDeclNode> {
  for leaf in tree {
    if leaf.node_type() == "FunctionDecl" {
      let func = mutate_leaf!(leaf, Box<FunctionDeclNode>);
      match &func.value {
        ExtraNodeData::String(func_name) => {
          if *func_name == name.to_string() {
            return Some(*func);
          }
        },
        _ => continue
      }
    }
  }
  None
}
