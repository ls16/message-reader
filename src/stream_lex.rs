use std::rc::Rc;
use js_sys::{Function, Uint8Array};
use wasm_bindgen::prelude::{JsValue};

use super::utils::*;
use super::lex::{LexBase, do_lex_action};
use super::dfa::{build, State, GotoStatesOpt};

pub struct StreamLex {
  buffer: Vec<u8>,
  preread: bool,
  preread_codes: Vec<u8>,
  cur_position: usize,
  states: Option<Rc<Vec<Option<State>>>>,
  goto_states: Option<Rc<GotoStatesOpt>>,
  on_tkn_data: Option<Function>,
  error: bool,
  is_pass: bool,
  state: usize,
  tkn_name: Option<usize>,
  tkn_value: Option<Vec<u8>>,
  action: Option<Rc<Function>>,
  push_tkn_name: Option<usize>,
  push_tkn_data_buffer: Option<Vec<u8>>,
  size: Option<usize>,
  stop_code: Option<u8>,
  w_term_name: usize
}

impl Clone for StreamLex {
  fn clone(&self) -> Self {
    Self {
      buffer: vec!(),
      preread: false,
      preread_codes: vec!(),
      cur_position: 0,
      states: self.states.clone(),
      goto_states: self.goto_states.clone(),
      on_tkn_data: self.on_tkn_data.clone(),
      error: false,
      is_pass: false,
      state: 0,
      tkn_name: None,
      tkn_value: None,
      action: None,
      push_tkn_name: None,
      push_tkn_data_buffer: None,
      size: None,
      stop_code: None,
      w_term_name: self.w_term_name
    }
  }
}

impl StreamLex {
  pub fn new(on_tkn_data: Option<Function>) -> Self {
    Self {
      buffer: vec!(),
      preread: false,
      preread_codes: vec!(),
      cur_position: 0,
      states: None,
      goto_states: None,
      on_tkn_data,
      error: false,
      is_pass: false,
      state: 0,
      tkn_name: None,
      tkn_value: None,
      action: None,
      push_tkn_name: None,
      push_tkn_data_buffer: None,
      size: None,
      stop_code: None,
      w_term_name: GrammarSymbol::w_term().name()
    }
  }
}

impl LexBase for StreamLex {
  fn init(&mut self) {
    self.buffer.clear();
    self.preread = false;
    self.preread_codes.clear();
    self.cur_position = 0;
    self.error = false;
    self.is_pass = false;
    self.state = 0;
    self.tkn_name = None;
    self.tkn_value = None;
    self.action = None;
    self.push_tkn_name = None;
    self.size = None;
    self.push_tkn_data_buffer = None;
  }

  fn box_clone(&self) -> Box<dyn LexBase> {
    Box::new(self.clone())
  }

  fn set_read_size(&mut self, tkn_name: usize, size: usize) {
    self.push_tkn_name = Some(tkn_name);
    self.size = Some(size);
    self.preread = false;
    self.state = 0;
    self.error = false;
    self.is_pass = false;
    self.tkn_name = None;
    self.tkn_value = None;
    self.stop_code = None;
  }

  fn set_read_to_code(&mut self, tkn_name: usize, code: u8) {
    self.push_tkn_name = Some(tkn_name);
    self.stop_code = Some(code);
    self.size = None;
  }

  fn set_on_tkn_data(&mut self, on_tkn_data: Option<Function>) {
    self.on_tkn_data = on_tkn_data;
  }

  fn set_data(&mut self, data: Vec<u8>) {
    self.buffer = data;
    self.cur_position = 0;
  }

  fn has_data(&self) -> bool {
    self.cur_position < self.buffer.len()
  }

  fn get_token(&mut self, exec_context: &JsValue) -> Result<Option<Token>, JsValue> {

    macro_rules! restore_state {
      ($is_pass: ident, $state: ident, $tkn_name: ident, $tkn_value: ident, $action: ident,
        $push_tkn_name: ident, $push_tkn_data_buffer: ident, $size: ident) => {
        $is_pass = self.is_pass;
        $state = self.state;
        $tkn_name = self.tkn_name;
        $tkn_value = match self.tkn_value.take() {
          Some(tkn_value) => tkn_value,
          _ => vec!()
        };
        $action = self.action.take();
        $push_tkn_name = self.push_tkn_name;
        $push_tkn_data_buffer = self.push_tkn_data_buffer.take();
        $size = self.size.take();
      }
    }

    macro_rules! save_state {
      ($is_pass: ident, $state: ident, $tkn_name: ident, $tkn_value: ident, $action: ident,
        $push_tkn_name: ident, $push_tkn_data_buffer: ident, $size: ident) => {
          self.is_pass = $is_pass;
          self.state = $state;
          self.tkn_name = $tkn_name;
          self.tkn_value = Some($tkn_value);
          self.action = $action;
          self.push_tkn_name = $push_tkn_name;
          self.push_tkn_data_buffer = $push_tkn_data_buffer;
          self.size = $size;
      }
    }

    const HIGHWATERMARK: usize = 64 * 1024;

    let mut is_pass;
    let mut tkn: Option<Token>;
    let mut state: usize;
    let mut code: Option<u8>;
    let mut tkn_name: Option<usize>;
    let tkn_value: Vec<u8>;
    let mut action: Option<Rc<Function>>;
    let push_tkn_name: Option<usize>;
    let push_tkn_data_buffer: Option<Vec<u8>>;
    let size: Option<usize>;

    restore_state!(is_pass, state, tkn_name, tkn_value, action, push_tkn_name, push_tkn_data_buffer, size);

    if let Some(size) = size {
      let mut size_to_end = size;
      let mut push_tkn_data_buffer = match push_tkn_data_buffer {
        Some(push_tkn_data_buffer) => push_tkn_data_buffer,
        _ => Vec::with_capacity(HIGHWATERMARK)
      };
      for _ in 0..self.preread_codes.len() {
        push_tkn_data_buffer.push(self.preread_codes.pop().unwrap());
        size_to_end -= 1;
        if size_to_end == 0 {break}
      }

      loop {
        if self.buffer.len() == 0 {
          let push_tkn_data_buffer = Some(push_tkn_data_buffer);
          let size = Some(size_to_end);
          save_state!(is_pass, state, tkn_name, tkn_value, None, push_tkn_name, push_tkn_data_buffer, size);
          return Ok(Some(Token::new(self.w_term_name, vec!())));
        }

        if size_to_end > (self.buffer.len() - self.cur_position) {
          push_tkn_data_buffer.extend_from_slice(&self.buffer[self.cur_position..]);
          size_to_end -= self.buffer.len() - self.cur_position;
          self.buffer.clear();
          self.cur_position = 0;
        } else {
          let new_position = self.cur_position + size_to_end;
          push_tkn_data_buffer.extend_from_slice(&self.buffer[self.cur_position..new_position]);
          if new_position < self.buffer.len() {
            self.cur_position = new_position;
          } else {
            self.buffer = vec!();
            self.cur_position = 0;
          }
          if let Some(ref on_tkn_data) = self.on_tkn_data {
            let push_tkn_name = JsValue::from(push_tkn_name.unwrap() as u32);
            let push_tkn_data = unsafe {Uint8Array::view(&push_tkn_data_buffer)};
            let end = JsValue::from(true);
            let _ = on_tkn_data.call3(exec_context, &push_tkn_name, &push_tkn_data, &end);
          }
          push_tkn_data_buffer.clear();
          return Ok(Some(Token::new(push_tkn_name.unwrap(), vec!())));
        }

        if push_tkn_data_buffer.len() > HIGHWATERMARK {
          if let Some(ref on_tkn_data) = self.on_tkn_data {
            let push_tkn_name = JsValue::from(push_tkn_name.unwrap() as u32);
            let push_tkn_data = unsafe {Uint8Array::view(&push_tkn_data_buffer)};
            let end = JsValue::from(false);
            let _ = on_tkn_data.call3(exec_context, &push_tkn_name, &push_tkn_data, &end);
          }
          push_tkn_data_buffer.clear();
        }
      }
    }

    loop {
      tkn = None;
      self.error = false;

      if self.preread_codes.len() == 0 {
        if self.buffer.len() == 0 {
          save_state!(is_pass, state, tkn_name, tkn_value, action, push_tkn_name, None, size);
          return Ok(Some(Token::new(self.w_term_name, vec!())))
        }
      }
      code = self.get_code();
      let mut tkn_value = tkn_value.clone();
      if state == 0 {
        if let Some(code) = code {
          tkn_value.push(code);
        }
      } else {
        self.preread = true;
        self.preread_codes.push(code.unwrap());
      }

      while code != None {
        match self.goto(state, code.unwrap() as usize) {
          Some(data) => {
            state = *data;
            if let Some(state_info) = self.state(state) {
              tkn_name = Some(state_info.accept());
              action = state_info.action().clone();
            }
            let goto_next_exists = self.goto_exists(state);
            if tkn_name.is_some() {
              if self.preread && self.preread_codes.len() > 0 {
                tkn_value.extend(self.preread_codes.clone());
                self.preread_codes.clear();
              }
              if goto_next_exists {self.preread = true}
              else {break}
            }
          },
          None => {
            if tkn_name.is_none() {
              self.error = true;
            }
            break;
          }
        }

        code = self.get_code();
        if code != None {
          let code = code.unwrap();
          if self.preread {
            self.preread_codes.push(code);
          }
          else {
            tkn_value.push(code);
          }
        }
      }

      if code == None {
        save_state!(is_pass, state, tkn_name, tkn_value, action, push_tkn_name, None, size);
        return Ok(Some(Token::new(self.w_term_name, vec!())))
      }

      if tkn_name.is_some() {
        let tkn_name = tkn_name.unwrap();
        let mut tkn_name_changed: Option<usize> = None;
        let mut tkn_name_changed1: Option<usize> = None;
        let mut tkn_value_changed: Option<Vec<u8>> = None;
        if let Some(ref func) = action {

          let get = || -> Vec<u8> {
            tkn_value.clone()
          };

          let mut set = |value: Vec<u8>| {
            tkn_value_changed = Some(value);
          };

          let mut set_name = |name: String| {
            tkn_name_changed = Some(hash(&name));
          };

          let mut set_name_from_hash = |hash_name: usize| {
            tkn_name_changed1 = Some(hash_name);
          };

          let mut pass = || {
            is_pass = true;
          };

          do_lex_action(&func, exec_context, &get, &mut set, &mut set_name, &mut set_name_from_hash, &mut pass)?;
  
          let tkn_name = match tkn_name_changed1 {
            Some(tkn_name_changed1) => tkn_name_changed1,
            _ => match tkn_name_changed {
              Some(tkn_name_changed) => tkn_name_changed,
              _ => tkn_name
            }
          };

          let tkn_value = match tkn_value_changed {
            Some(tkn_value_changed) => tkn_value_changed,
            _ => tkn_value
          };

          tkn = Some(Token::new(tkn_name, tkn_value));
        } else {
          tkn = Some(Token::new(tkn_name, tkn_value));
        }
      }

      if !is_pass {break}
      state = 0;
      is_pass = false;
      tkn_name = None;
    }

    self.state = 0;
    self.tkn_name = None;
    Ok(tkn)
  }
}

impl StreamLex {
  pub fn set_regular_definition_text(&mut self, reg_exp: String) -> Result<(), String> {
    let (states, goto_states) = build(reg_exp);
    self.states = Some(Rc::new(states));
    self.goto_states = Some(Rc::new(goto_states));
    Ok(())
  }

  pub fn set_states(&mut self, states: Option<Rc<Vec<Option<State>>>>, goto_states: Option<Rc<GotoStatesOpt>>) {
    self.states = states;
    self.goto_states = goto_states;
  }

  fn state(&self, state: usize) -> Option<&State> {
    match self.states {
      Some(ref states) => match states.get(state) {
        Some(state) => match state {
          Some(state) => Some(state),
          _ => None
        },
        _ => None
      },
      None => None
    }
  }

  fn goto(&self, state: usize, code: usize) -> Option<&usize> {
    match self.goto_states {
      Some(ref goto_states) => goto_states.state(state, code),
      None => None
    }
  }

  fn goto_exists(&self, state: usize) -> bool {
    match self.goto_states {
      Some(ref goto_states) => goto_states.state_exists(state),
      None => false
    }
  }

  fn get_code(&mut self) -> Option<u8> {
    if self.preread_codes.len() > 0 {return Some(self.preread_codes.pop().unwrap())}
    if self.buffer.len() == 0 {return None}
    let code = self.buffer[self.cur_position];
    self.cur_position += 1;
    if self.cur_position > (self.buffer.len() - 1) {
      self.buffer.clear();
      self.cur_position = 0;
    }
    Some(code)
  }
}
