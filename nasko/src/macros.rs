//! General macros for the Nasko Compiler

/// Generates a compile error
///
/// Currently, it prints the error and panics
pub fn compile_error(error: String) {
  print!("\n\n");
  println!("{}", error);
  panic!("");
}

/// Literal macro
#[macro_export]
macro_rules! literal {
  ($tree:ident, $state:ident, $lex:ident, $expression:expr) => {
    if $state.in_expression {
      let expr_err = format!("Unmatched expression before span: {:?}", $lex.span());
      if let Some(leaf) = $tree.children.pop() {
        if leaf.node_type() != "BinaryExpression" {
          compile_error(expr_err);
          continue;
        }

        let mut expr = mutate_leaf!(leaf, Box<BinaryExpression>);
        expr.lhs = Some(($expression)());
        $tree.push_leaf(expr);
        $state.in_expression = false;
        continue;
      }
      compile_error(expr_err);
    } else if $state.in_return {
      let mut func = $tree.children.pop().unwrap();
      let ret = func.get_leaf_mut(func.get_leaves().len() - 1).unwrap();
      
      ret.push_leaf(($expression)());

      $tree.push_leaf(func);
    } else if $state.has_unknown_ident && $state.in_parens {
      /*
        ValueNode {
          ta: NaskoType::Unknown,
          ntype: "UnknownIdent".to_string(),
          value: ExtraNodeData::String(ident),
          children: vec![]
        }
      */
      let mut fd = pop_leaf!($tree.children, Box<FunctionDeclNode>);
      let mut id = pop_leaf!(fd.children, Box<ValueNode>);
      if id.ntype.as_str() == "UnknownIdent" {
        id.ntype = "CallExpression".to_string();
      }
      id.push_leaf(($expression)());

      fd.push_leaf(id);
      $tree.push_leaf(fd);
    } else {
      $tree.push_leaf(($expression)());
    }
  };
}

/// Expression push macro
#[macro_export]
macro_rules! expr_push {
  ($tree:ident, $state:ident, $lex:ident, $expression:expr) => {
    if let Some(c) = $tree.children.pop() {
      ($expression)(c);
      $state.in_expression = true;
    } else {
      crate::macros::compile_error(format!("Right hand side missing in add expression ({:?}):\n\t{}", $lex.span(), $lex.slice()));
    }
  };
}

/// Mutate a leaf (usually used by `pop_leaf!`)
#[macro_export]
macro_rules! mutate_leaf {
  ($leaf:expr, $ts_type:ty) => { unsafe {
    transmute::transmute::<Box<dyn ASTNode>, $ts_type>($leaf)
  } };
}

/// Pop a leaf from the leaf stack and mutate it
#[macro_export]
macro_rules! pop_leaf {
  ($leaves:expr, $ts_type:ty) => {
    mutate_leaf!($leaves.pop().unwrap(), $ts_type)
  };
}
