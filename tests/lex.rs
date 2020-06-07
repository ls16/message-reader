use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

use server::utils::*;
use server::lex::*;

mod fixtures;
use fixtures::*;

#[wasm_bindgen_test]
fn test_regular_definition_text() {
  let mut lex = Lex::new("".to_string());
  lex.set_regular_definition_text(reg_exp());
  assert_eq!(lex.rules().len(), 7, "Invalid number of rules");
  let mut iter = lex.rules().iter();
  assert_eq!(iter.find(|rule| rule.name() == hash("id")).unwrap().action().is_some(), false, "Found action for 'id' rule");
  assert_eq!(iter.find(|rule| rule.name() == hash("plus")).unwrap().action().is_some(), true, "Not found action for 'plus' rule");
  assert_eq!(iter.find(|rule| rule.name() == hash("digit")).is_none(), true, "Not found rule 'digit'");
  assert_eq!(iter.find(|rule| rule.name() == hash("letter")).is_none(), true, "Not found rule 'letter'");
}

#[wasm_bindgen_test]
fn test_get_token() {
  let null_context = &JsValue::NULL;
  let mut lex = Lex::new(lex_text());
  lex.set_regular_definition_text(reg_exp());
  let tkn = lex.get_token(null_context).expect("Error in get_token").unwrap();
  assert_eq!(tkn.name(), hash("id"), "Invalid token name");
  assert_eq!(tkn.value_to_string(), "name123", "Invalid token value");
  let tkn = lex.get_token(null_context).expect("Error in get_token").unwrap();
  assert_eq!(tkn.name(), hash("space"), "Invalid token name");
  let tkn = lex.get_token(null_context).expect("Error in get_token").unwrap();
  assert_eq!(tkn.name(), hash("number"), "Invalid token name");
  assert_eq!(tkn.value_to_string(), "456", "Invalid token value");
  let tkn = lex.get_token(null_context).expect("Error in get_token").unwrap();
  assert_eq!(tkn.name(), hash("space"), "Invalid token name");
  let tkn = lex.get_token(null_context).expect("Error in get_token").unwrap();
  assert_eq!(tkn.name(), hash("("), "Invalid token name");
  let tkn = lex.get_token(null_context).expect("Error in get_token").unwrap();
  assert_eq!(tkn.name(), hash("number"), "Invalid token name");
  assert_eq!(tkn.value_to_string(), "75", "Invalid token value");
  let tkn = lex.get_token(null_context).expect("Error in get_token").unwrap();
  assert_eq!(tkn.name(), hash("+"), "Invalid token name");
  let tkn = lex.get_token(null_context).expect("Error in get_token").unwrap();
  assert_eq!(tkn.name(), hash("number"), "Invalid token name");
  assert_eq!(tkn.value_to_string(), "86", "Invalid token value");
  let tkn = lex.get_token(null_context).expect("Error in get_token").unwrap();
  assert_eq!(tkn.name(), hash("*"), "Invalid token name");
  let tkn = lex.get_token(null_context).expect("Error in get_token").unwrap();
  assert_eq!(tkn.name(), hash("number"), "Invalid token name");
  assert_eq!(tkn.value_to_string(), "7", "Invalid token value");
  let tkn = lex.get_token(null_context).expect("Error in get_token").unwrap();
  assert_eq!(tkn.name(), hash(")"), "Invalid token name");
}
