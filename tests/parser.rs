use wasm_bindgen_test::*;

use server::lex::*;
use server::parser::*;
use server::dfa_grammar::*;

mod fixtures;

#[wasm_bindgen_test]
fn test_parse() {
  let ctx = ExecContext::new();
  let js_ctx = cast_exec_context_to_js_value(0, ctx);
  let text = "(1+2*a14)";
  let mut lex = Lex::new(text.to_string());
  lex.set_regular_definition_text(fixtures::reg_exp());

  let mut parser = Parser::new(Box::new(lex));
  let _ = parser.set_grammar(fixtures::grammar());
  parser.disable_state_logging();

  let res = parser.parse(&js_ctx);
  let is_ok = match res {
    Ok(res) => res == ParseResult::ParseSuccess,
    _ => false
  };
  assert_eq!(is_ok, true, "Invalid parsed: {:?}", text);
}