use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

use server::utils::*;
use server::lex::*;
use server::stream_lex::*;

mod fixtures;
use fixtures::*;

#[wasm_bindgen_test]
async fn test_regular_definition_text() {
  let null_context = &JsValue::NULL;
  let data: Vec<u8> = vec!(72, 84, 84, 80, 226, 157, 164, 71, 69, 84);

  let mut lex = StreamLex::new(None);
  let _ = lex.set_regular_definition_text(regular_definition_text1());
  lex.set_data(data);

  let w_term = GrammarSymbol::w_term();

  let tkn = lex.get_token(null_context).expect("Error in get_token").expect("No token");
  assert_eq!(tkn.value() == &vec!(72u8, 84, 84, 80, 226, 157, 164), true, "Invalid token value: {:?}", tkn.value());

  let tkn = lex.get_token(null_context).expect("Error in get_token").expect("No token");
  assert_eq!(tkn.value() == &vec!(71u8, 69, 84), true, "Invalid token value: {:?}", tkn.value());

  let tkn = lex.get_token(null_context).expect("Error in get_token").expect("No token");
  assert_eq!(tkn.name() == w_term.name(), true, "Invalid token value: {:?}", tkn.name());
}
