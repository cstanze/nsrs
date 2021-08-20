mod search;

use crate::ast::*;
use crate::semantics::*;
use crate::macros::compile_error;

use self::search::func_with_name;

pub fn annotate_types(tree: &mut SourceNode) {
  let main_function = func_with_name("main", tree.children.clone());
  
  match main_function {
    Some(mut main_function) => {
      if main_function.ta != NaskoType::Number {
        compile_error(format!("`main` function must be annotated with the return value: int"))
      }

      match main_function.params.len() {
        0 => {}, // Ignore if there aren't any arg params
        1 | 2 => check_main_args(&mut main_function.params),
        _ => compile_error(format!("Only 2 arguments allowed in the main function: argc (int), and argv (string[])"))
      }
    }
    None => {
      compile_error(format!("Missing `main` function from source. Please implement a main function"));
    }
  }
}

fn check_main_args(params: &mut Vec<Box<ValueNode>>) {
  println!("checking params: {:?}", params);
  let double = params.len() == 2;
  let mut err = String::from("");

  if params[0].ta != NaskoType::Number && params[0].ta != NaskoType::Unknown {
    err.push_str(&format!("Invalid type for main argument ({:?})\nExpected type {:?} instead found type: {:?}\n", params[0].value, NaskoType::Number, params[0].ta));
  }
  if double {
    if params[1].ta != NaskoType::String && params[1].ta != NaskoType::Unknown {
      err.push_str(&format!("Invalid type for main argument ({:?})\nExpected type {:?} instead found type: {:?}", params[1].value, NaskoType::String, params[1].ta));
    }
  }

  if err != "".to_string() {
    compile_error(err);
  }

  // Annotate argument types if unknown
  if params[0].ta == NaskoType::Unknown {
    params[0].ta = NaskoType::Number;
  }
  if double {
    if params[1].ta == NaskoType::Unknown {
      params[1].ta = NaskoType::String;
    }
  }
}
