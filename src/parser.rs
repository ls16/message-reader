use std::rc::Rc;
use std::fmt::Debug;
use js_sys::{Function};
use wasm_bindgen::prelude::{JsValue, wasm_bindgen};

use super::utils::*;
use super::lex::*;
use super::lalr::*;

#[wasm_bindgen]
pub enum ParserType {
  LR1,
  LALR1
}

#[derive(PartialEq, Debug)]
pub enum ParseResult {
  ParseWait,
  ParseSuccess
}

#[derive(Debug, Clone)]
pub struct StackItem {
  state: usize,
  bind_id: Option<usize>,
  symbol: Option<GrammarSymbol>
}

impl StackItem {
  pub fn new(state: usize, bind_id: Option<usize>, symbol: Option<GrammarSymbol>) -> Self {
    Self {
      state,
      bind_id,
      symbol
    }
  }
}

pub struct Stack (Vec<StackItem>);

#[derive(Debug, Clone)]
struct Next {
  name: usize,
  insert_name: Option<usize>,
  insert_value: Option<Vec<u8>>,
  size: Option<usize>,
  stop_code: Option<u8>,
  last_name: Option<usize>
}

impl Next {
  pub fn new(name: usize, insert_name: Option<usize>, insert_value: Option<Vec<u8>>,
    size: Option<usize>, stop_code: Option<u8>) -> Self {
    Self {
      name,
      insert_name,
      insert_value,
      size,
      stop_code,
      last_name: None
    }
  }

  pub fn name(&self) -> usize {
    self.name
  }

  pub fn insert_name(&self) -> &Option<usize> {
    &self.insert_name
  }

  pub fn insert_value(&self) -> &Option<Vec<u8>> {
    &self.insert_value
  }
}

#[derive(Debug)]
pub struct Parser {
  lex: Box<dyn LexBase>,
  goto_states: Rc<GotoStatesOpt>,
  action_state: Option<ActionState2>,
  action_states: Rc<ActionStatesOpt>,
  grammar: Rc<Grammar>,
  nexts: Vec<Next>,
  stack: Option<Vec<StackItem>>,
  cur_symbol: Option<GrammarSymbol>,
  is_e_symbol: bool,
  state_logging: bool
}

impl Clone for Parser {
  fn clone(&self) -> Self {
    Self {
      lex: self.lex.box_clone(),
      goto_states: self.goto_states.clone(),
      action_state: None,
      action_states: self.action_states.clone(),
      grammar: self.grammar.clone(),
      nexts: vec!(),
      stack: Some(vec!()),
      cur_symbol: Some(GrammarSymbol::s_term()),
      is_e_symbol: false,
      state_logging: self.state_logging
    }
  }
}

impl Parser {
  pub fn new(lex: Box<dyn LexBase>) -> Self {
    Self {
      lex,
      goto_states: Rc::new(GotoStatesOpt::new()),
      action_state: None,
      action_states: Rc::new(ActionStatesOpt::new()),
      grammar: Rc::new(Grammar::new()),
      nexts: vec!(),
      stack: Some(vec!()),
      cur_symbol: Some(GrammarSymbol::s_term()),
      is_e_symbol: false,
      state_logging: true
    }
  }

  pub fn init(&mut self) {
    self.lex.init();
    self.nexts.clear();
    self.stack = Some(vec!());
    self.cur_symbol = Some(GrammarSymbol::s_term());
    self.is_e_symbol = false;
  }

  pub fn enable_state_logging(&mut self) {
    self.state_logging = true;
  }

  pub fn disable_state_logging(&mut self) {
    self.state_logging = false;
  }

  pub fn set_grammar(&mut self, grammar: String, parser_type: ParserType) -> Result<(), String> {
    let grammar = GrammarBuilder::from_text(grammar).unwrap();
    let goto_states;
    let action_states;
    match parser_type {
      ParserType::LALR1 => {
        goto_states = LALRBuilder::build_goto_states(&grammar);
        action_states = LALRBuilder::build_action_states(&grammar, &goto_states)?;
      },
      ParserType::LR1 => {
        goto_states = LRBuilder::build_goto_states(&grammar);
        action_states = LRBuilder::build_action_states(&grammar, &goto_states)?;
      }
    }

    self.grammar = Rc::new(grammar);
    self.goto_states = Rc::new(goto_states);
    self.action_states = Rc::new(action_states);
    Ok(())
  }

  pub fn set_states(&mut self, grammar: Rc<Grammar>, goto_states: Rc<GotoStatesOpt>, action_states: Rc<ActionStatesOpt>) {
    self.grammar = grammar;
    self.goto_states = goto_states;
    self.action_states = action_states;
  }

  pub fn set_text(&mut self, value: String) {
    self.lex.set_text(value);
  }

  pub fn set_data(&mut self, data: Vec<u8>) {
    self.lex.set_data(data);
  }

  pub fn has_data(&self) -> bool {
    self.lex.has_data()
  }

  pub fn data(&self) -> Vec<u8> {
    self.lex.data()
  }

  pub fn set_on_tkn_data(&mut self, on_tkn_data: Option<Function>) {
    self.lex.set_on_tkn_data(on_tkn_data);
  }

  pub fn parse(&mut self, exec_context: &JsValue) -> Result<ParseResult, JsValue> {
    let e_term = GrammarSymbol::e_term();
    let w_term = GrammarSymbol::w_term();
    let rust_action_name = &hash("rust_action");
    let action_name = &hash("action");

    macro_rules! get_symbol {
      ($exec_context: ident) => {
        {
          let mut res: Option<GrammarSymbol> = None;
          if self.nexts.len() > 0 {
            let next = &self.nexts[0];
            let mut is_equal_last_name = false;
            if let Some(ref last_name) = next.last_name {
              is_equal_last_name = next.name() == *last_name;
            }
            if next.name() == e_term.name() || is_equal_last_name {
              if next.size.is_some() {
                self.lex.set_read_size(next.insert_name().unwrap(), next.size.unwrap());
              } else {
                let symbol = match next.insert_name() {
                  Some(insert_name) => GrammarSymbol::term(*insert_name, next.insert_value().clone()),
                  _ => GrammarSymbol::s_term()
                };
                res = Some(symbol);
              }
              self.nexts.remove(0);
            }
          }

          if res.is_some() {
            res.unwrap()
          } else {
            let tkn = self.lex.get_token(exec_context)?;
            let symbol = GrammarSymbol::from_token(tkn);
            if self.nexts.len() > 0 {
              if let Some(ref mut next) = self.nexts.get_mut(0) {
                next.last_name = Some(symbol.name());
              }
            }
            symbol
          }
        }
      }
    }

    macro_rules! restore_state {
      ($stack: ident) => {
        $stack = self.stack.take().unwrap();
      }
    }

    macro_rules! save_state {
      ($stack: ident) => {
        self.stack = Some($stack);
      }
    }

    macro_rules! clear_state {
      ($stack: ident) => {
        $stack.clear();
        self.stack = Some($stack);
        self.cur_symbol = Some(GrammarSymbol::s_term());
        self.is_e_symbol = false;
      }
    }

    fn err() -> Result<ParseResult, JsValue> {
      Err(JsValue::from("Error parse"))
    }

    let mut stack: Vec<StackItem>;
    let mut cur_symbol;
    let mut is_e_symbol;
    restore_state!(stack);

    cur_symbol = get_symbol!(exec_context);
    if cur_symbol.name() == w_term.name() {
      save_state!(stack);
      return Ok(ParseResult::ParseWait);
    }
    is_e_symbol = cur_symbol.name() == e_term.name();

    if stack.len() == 0 {
      stack.push(StackItem::new(0, None, None));
    }

    loop {
      let stack_item = stack.last();
      if stack_item.is_none() {return err()}
      let stack_item = stack_item.unwrap();
      let mut action = self.action_states.state(stack_item.state, cur_symbol.name());
      if action.is_none() {
        is_e_symbol = true;
        action = self.action_states.state(stack_item.state, e_term.name());
      }
      if action.is_none() {return err()}
      let action = action.unwrap();
      match action.state() {
        ActionState::Shift => {
          let symbol;
          match is_e_symbol {
            false => {
              symbol = Some(cur_symbol.clone());
            },
            true => {
              symbol = Some(e_term.clone());
            }
          };

          if cfg!(debug_assertions) {
            if self.state_logging {
              match symbol {
                Some(ref symbol) => {
                  log(format!("Shift,  symbol name: {:?}, symbol value: {:?}",
                      get_original_name(symbol.name()), symbol.value()).as_str());
                }
                _ => {}
              }
            }
          }

          let goto = action.goto();
          if goto.is_none() {return err()}
          let goto = goto.unwrap();
          stack.push(StackItem::new(goto, None, symbol));
          if !is_e_symbol {
            cur_symbol = get_symbol!(exec_context);
            if cur_symbol.name() == w_term.name() {
              save_state!(stack);
              return Ok(ParseResult::ParseWait);
            }
          }
          is_e_symbol = false;
        },
        ActionState::Reduce => {
          if let Some(action_prod) = action.production() {
            let mut new_symbol = GrammarSymbol::non_term(action_prod.name(), None);
            let mut new_symbol_name: Option<usize> = None;
            let mut new_symbol_name1: Option<usize> = None;
            let mut new_symbol_val: Option<Vec<u8>> = None;
            let mut bind_id: Option<usize> = None;
            let mut nexts: Option<Vec<Next>> = None;

            let action = action_prod.attr(rust_action_name);
            if let Some(action) = action {
              if let Some(action) = action.as_rust_action() {

                let mut set = |index: usize, index2: Option<usize>, index3: Option<usize>,
                index4: Option<usize>, index5: Option<usize>| {
                  let idx = stack.len() - 1 - index;
                  match stack.get_mut(idx) {
                    Some(item) => {
                      if let Some(ref mut symbol) = item.symbol {
                        new_symbol.set_value(symbol.take_value());
                      } else {
                        new_symbol.set_value(None);
                      }
                    },
                    None => {new_symbol.set_value(None);}
                  };

                  if let Some(index2) = index2 {
                    let idx = stack.len() - 1 - index2;
                    if let Some(item) = stack.get_mut(idx) {
                      if let Some(ref mut symbol) = item.symbol {
                        new_symbol.extend_value(symbol.take_value());
                      }
                    }
                  }

                  if let Some(index3) = index3 {
                    let idx = stack.len() - 1 - index3;
                    if let Some(item) = stack.get_mut(idx) {
                      if let Some(ref mut symbol) = item.symbol {
                        new_symbol.extend_value(symbol.take_value());
                      }
                    }
                  }

                  if let Some(index4) = index4 {
                    let idx = stack.len() - 1 - index4;
                    if let Some(item) = stack.get_mut(idx) {
                      if let Some(ref mut symbol) = item.symbol {
                        new_symbol.extend_value(symbol.take_value());
                      }
                    }
                  }

                  if let Some(index5) = index5 {
                    let idx = stack.len() - 1 - index5;
                    if let Some(item) = stack.get_mut(idx) {
                      if let Some(ref mut symbol) = item.symbol {
                        new_symbol.extend_value(symbol.take_value());
                      }
                    }
                  }
                };

                match action.command() {
                  RustActionCommand::Set => {
                    set(action.index(), action.index2(), action.index3(), action.index4(), action.index5());
                  },
                }
              }
            } else {

              let action = action_prod.attr(action_name);
              if let Some(action) = action {
                if let Some(func) = action.as_function() {
                  let mut bind = |id: usize| {
                    bind_id = Some(id);
                  };

                  let id = |index: usize| -> Option<usize> {
                    match stack.get(stack.len() - 1 - index) {
                      Some(item) => {
                        item.bind_id
                      },
                      None => None
                    }
                  };

                  let lookup = || -> Option<Vec<u8>> {
                    cur_symbol.value().clone()
                  };

                  let get = |index: usize| -> Option<Vec<u8>> {
                    match stack.get(stack.len() - 1 - index) {
                      Some(item) => {
                        if let Some(ref symbol) = item.symbol {
                          symbol.value().clone()
                        } else {
                          None
                        }
                      },
                      None => None
                    }
                  };

                  let mut set = |index: usize| {
                    match stack.get(stack.len() - 1 - index) {
                      Some(item) => {
                        if let Some(ref symbol) = item.symbol {
                          new_symbol.set_value(symbol.value().clone());
                        } else {
                          new_symbol.set_value(None);
                        }
                      },
                      None => {new_symbol.set_value(None);}
                    }
                  };

                  let mut set_val = |value: Vec<u8>| {
                    new_symbol_val = Some(value);
                  };

                  let mut set_name = |name: String| {
                    new_symbol_name = Some(hash(&name));
                  };

                  let mut set_name_from_hash = |hash_name: usize| {
                    new_symbol_name1 = Some(hash_name);
                  };

                  let mut push_after = |name: String, insert_name: Option<String>,
                    insert_value: Option<Vec<u8>>, size: Option<usize>, stop_code: Option<u8>| {
                    let insert_name = match insert_name {
                      Some(ref insert_name) => Some(hash(insert_name)),
                      None => None
                    };
                    let mut set_params_count = 0;
                    if insert_value.is_some() {set_params_count += 1;}
                    if size.is_some() {set_params_count += 1;}
                    if stop_code.is_some() {set_params_count += 1;}
                    if set_params_count > 1 {panic!("Only one parameter ('insert_value', 'size' or 'stop_code') can be set at a time");}
                    let next = Next::new(hash(&name), insert_name, insert_value, size, stop_code);
                    match nexts {
                      Some(ref mut nexts) => nexts.push(next),
                      _ => {
                        let mut _nexts: Vec<Next> = vec!();
                        _nexts.push(next);
                        nexts = Some(_nexts);
                      }
                    }
                  };

                  do_parser_action(&func, exec_context, &mut bind, &id, &lookup, &get, &mut set, &mut set_val, &mut set_name, &mut set_name_from_hash, &mut push_after)?;

                  if new_symbol_val.is_some() {
                    new_symbol.set_value(new_symbol_val);
                  }

                  if new_symbol_name1.is_some() {
                    new_symbol.set_name(new_symbol_name1.unwrap());
                  } else if new_symbol_name.is_some() {
                    new_symbol.set_name(new_symbol_name.unwrap());
                  }

                  if let Some(nexts) = nexts {
                    self.nexts.extend(nexts);
                  }
                }
              }
            }
  
            stack.truncate(stack.len() - action_prod.len());
  
            let stack_item = stack.last();
            if stack_item.is_none() {return err()}
            let stack_item = stack_item.unwrap();
            let new_state = self.goto_states.state(stack_item.state, new_symbol.name());
            if new_state.is_none() {return err()}
            let new_state = new_state.unwrap();
  
            if cfg!(debug_assertions) {
              if self.state_logging {
                log(format!("Reduce, production name: {:?}", get_original_name(action_prod.name())).as_str());
              }
            }

            stack.push(StackItem::new(*new_state, bind_id, Some(new_symbol)));
          } else {
            return err();
          }
        },
        ActionState::Accept => {
          clear_state!(stack);
          return Ok(ParseResult::ParseSuccess);
        }
      }
    }
  }
}

#[wasm_bindgen(module = "/src/parser.js")]
extern "C" {
  #[wasm_bindgen(catch)]
  fn do_parser_action(action: &Function, context: &JsValue, bind: &mut dyn FnMut(usize), id: &dyn Fn(usize) -> Option<usize>,
    lookup: &dyn Fn() -> Option<Vec<u8>>, get: &dyn Fn(usize) -> Option<Vec<u8>>,
    set: &mut dyn FnMut(usize), set_val: &mut dyn FnMut(Vec<u8>),
    set_name: &mut dyn FnMut(String), set_name_from_hash: &mut dyn FnMut(usize),
    push_after: &mut dyn FnMut(String, Option<String>, Option<Vec<u8>>, Option<usize>, Option<u8>)
  ) -> Result<(), JsValue>;
}