use std::rc::Rc;
use js_sys::{Object, Reflect, Function, Uint8Array};
use wasm_bindgen::prelude::{JsValue, wasm_bindgen};

use super::lalr::{GrammarBuilder, StatesBuilder, LALRBuilder, LRBuilder};
use super::parser::{Parser, ParseResult, ParserType};
use super::dfa::build;
use super::stream_lex::StreamLex;

#[wasm_bindgen]
pub struct Executor {
  parser: Parser,
  exec_context: Option<Object>
}

impl Clone for Executor {
  fn clone(&self) -> Self {
    Executor {
      parser: self.parser.clone(),
      exec_context: None
    }
  }
}

#[wasm_bindgen]
impl Executor {
  pub fn clone_executor(&self) -> Executor {
    self.clone()
  }

  pub fn build(reg_exp: String, grammar: String, parser_type: ParserType) -> Self {
    let (lex_states, lex_goto_states) = build(reg_exp);
    let lex_states = Some(Rc::new(lex_states));
    let lex_goto_states = Some(Rc::new(lex_goto_states));
    let on_tkn_data = None;
    let mut lex = StreamLex::new(on_tkn_data);
    lex.set_states(lex_states, lex_goto_states);

    let grammar = GrammarBuilder::from_text(grammar).unwrap();
    let goto_states;
    let action_states;
    match parser_type {
      ParserType::LALR1 => {
        goto_states = LALRBuilder::build_goto_states(&grammar);
        action_states = LALRBuilder::build_action_states(&grammar, &goto_states).unwrap();
      },
      ParserType::LR1 => {
        goto_states = LRBuilder::build_goto_states(&grammar);
        action_states = LRBuilder::build_action_states(&grammar, &goto_states).unwrap();
      }
    }
    let parser_grammar = Rc::new(grammar);
    let parser_goto_states = Rc::new(goto_states);
    let parser_action_states = Rc::new(action_states);

    let mut parser = Parser::new(Box::new(lex));
    parser.set_states(parser_grammar, parser_goto_states, parser_action_states);

    Executor {
      parser,
      exec_context: None
    }
  }

  pub fn parse_init(&mut self) {
    self.parser.init();
  }

  pub fn has_data(&self) -> bool {
    self.parser.has_data()
  }

  pub fn data(&self) -> Vec<u8> {
    self.parser.data()
  }

  pub fn parse_data(&mut self, data: &Uint8Array, proto: &Object,
    socket_key: &JsValue, socket: &JsValue,
    on_before_parse: Option<Function>, on_after_parse: Option<Function>,
    on_tkn_data: Option<Function>) -> Result<(), JsValue> {
    self.parser.set_on_tkn_data(on_tkn_data);
    self.parser.set_data(data.to_vec());
    loop {
      let exec_context = match self.exec_context.take() {
        Some(exec_context) => exec_context,
        None => {
          let exec_context = Object::create(proto);
          let _ = Reflect::set(&exec_context, socket_key, socket);
          if let Some(ref on_before_parse) = on_before_parse {
            let _ = on_before_parse.call0(&exec_context);
          };
          exec_context
        }
      };

      let result = self.parser.parse(&exec_context)?;
      if result == ParseResult::ParseWait {
        self.exec_context = Some(exec_context);
        break;
      };

      if let Some(ref on_after_parse) = on_after_parse {
        let break_parse = on_after_parse.call0(&exec_context)?;
        if break_parse == JsValue::TRUE {break};
      }

      if !self.parser.has_data() {break}
    }
    Ok(())
  }

}
