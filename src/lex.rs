use std::collections::HashMap;
use std::fmt;
use std::fmt::Debug;
use js_sys::Function;
use regex::Regex;
use wasm_bindgen::prelude::{JsValue, wasm_bindgen};

use super::utils::*;

struct LexRuleIntl {
  expression: String,
  action: Option<Function>,
  define: bool,
  position: usize
}

impl LexRuleIntl {
  fn new(expression: &str, action: Option<Function>, define: bool, position: usize) -> Self {
    LexRuleIntl {
      expression: String::from(expression),
      action,
      define,
      position
    }
  }
}

#[derive(Debug)]
pub struct LexRule {
  name: usize,
  expression: Regex,
  action: Option<Function>,
  define: bool,
  position: usize
}

impl LexRule {
  pub fn new(name: usize, expression: Regex, action: Option<Function>, define: bool, position: usize) -> Self {
    LexRule {
      name,
      expression,
      action,
      define,
      position
    }
  }

  pub fn name(&self) -> usize {
    self.name
  }

  pub fn expression(&self) -> &Regex {
    &self.expression
  }

  pub fn action(&self) -> &Option<Function> {
    &self.action
  }

  pub fn define(&self) -> bool {
    self.define
  }
}

pub trait LexBase {
  fn init(&mut self) {
    unimplemented!();
  }

  fn box_clone(&self) -> Box<dyn LexBase> {
    unimplemented!();
  }

  #[allow(unused_variables)]
  fn set_text(&mut self, value: String) {
    unimplemented!();
  }

  #[allow(unused_variables)]
  fn set_read_size(&mut self, tkn_name: usize, size: usize) {
    unimplemented!();
  }

  #[allow(unused_variables)]
  fn set_read_to_code(&mut self, tkn_name: usize, stop_code: u8) {
    unimplemented!();
  }

  #[allow(unused_variables)]
  fn set_data(&mut self, data: Vec<u8>) {
    unimplemented!();
  }

  #[allow(unused_variables)]
  fn has_data(&self) -> bool {
    unimplemented!();
  }

  #[allow(unused_variables)]
  fn data(&self) -> Vec<u8> {
    unimplemented!();
  }

  #[allow(unused_variables)]
  fn set_on_tkn_data(&mut self, on_tkn_data: Option<Function>) {
    unimplemented!();
  }

  fn get_token(&mut self, exec_context: &JsValue) -> Result<Option<Token>, JsValue>;
}

impl Debug for dyn LexBase {
  fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
    Ok(())
  }
}

pub struct Lex {
  text: String,
  cur_position: usize,
  rules: Vec<LexRule>,
  reg_exp: String,
  error: bool
}

impl LexBase for Lex {
  fn set_text(&mut self, value: String) {
    self.text = value;
    self.cur_position = 0;
  }

  fn get_token(&mut self, exec_context: &JsValue) -> Result<Option<Token>, JsValue> {
    self.error = false;
    let mut is_pass = false;
    let mut tkn: Option<Token> = None;
    loop {
      if self.cur_position < self.text.len() {
        let cur_text = &self.text[self.cur_position..];
        let mut tkn_name: Option<usize> = None;
        let mut tkn_value: Option<Vec<u8>> = None;
        let mut pos: Option<usize> = None;
        for i in 0..self.rules.len() {
          let rule = &self.rules[i];
          let result = rule.expression.find(cur_text);
          if result == None {continue};
          let result = result.unwrap();
          if result.start() > 0 {continue};
          let res_len = result.end() - result.start();
          match tkn_value {
            Some(ref val) if res_len > val.len() => {
              tkn_name = Some(rule.name.clone());
              tkn_value = Some(Vec::from(cur_text[..result.end()].as_bytes()));
              pos = Some(i);
            },
            None => {
              tkn_name = Some(rule.name.clone());
              tkn_value = Some(Vec::from(cur_text[..result.end()].as_bytes()));
              pos = Some(i);
            },
            _ => {}
          }
        }
        if tkn_name.is_some() {
          let tkn_name = tkn_name.unwrap();
          let tkn_value = tkn_value.unwrap();
          let mut tkn_name_changed: Option<usize> = None;
          let mut tkn_name_changed1: Option<usize> = None;
          let mut tkn_value_changed: Option<Vec<u8>> = None;
          self.cur_position += tkn_value.len();
          let action = self.rules[pos.unwrap()].action();
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
  
            let _ = do_lex_action(&func, exec_context, &get, &mut set, &mut set_name, &mut set_name_from_hash, &mut pass);
  
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
        self.error = tkn == None;
      }
  
      if !is_pass {break}
      is_pass = false;
    }

    Ok(tkn)
  }
}

impl Lex {
  pub fn new(text: String) -> Self {
    Lex {
      text,
      cur_position: 0,
      rules: Vec::new(),
      reg_exp: String::new(),
      error: false
    }
  }

  pub fn text(&self) -> &String {
    &self.text
  }

  pub fn rules(&self) -> &Vec<LexRule> {
    &self.rules
  }

  pub fn set_regular_definition_text(&mut self, value: String) {
    self.cur_position = 0;
    self.reg_exp = String::new();
    self.rules.clear();

    if value == "" {return;}

    let mut rules: HashMap<usize, LexRuleIntl> = HashMap::new();

    let rg_name = Regex::new(r"((_|[A-Za-z])(_|[A-Za-z]|[0-9])*)|('\S+')").unwrap();
    let rg_expression = Regex::new(r"\S+").unwrap();
    let rg_def = Regex::new(r"DEF").unwrap();
    let rg_action = Regex::new(r"\{(\S|\s)*\}").unwrap();

    for item in value.split("\n") {
      let result = rg_name.find(item);
      if result == None {continue};
      let result = result.unwrap();
      let name = match &item[result.start()..result.start()+1] {
        "\'" => &item[result.start()+1..result.end()-1],
        _ => &item[result.start()..result.end()]
      };
      let name = hash(&(name.to_string()));
      if rules.contains_key(&name) {panic!("Rule '{}' already defined", name)};
      let mut index = result.end() + 1;
      let result = rg_expression.find(&item[index..]);
      let result = result.expect(format!("Expression for '{}' is not defined", name).as_str());
      index += result.start();
      let len = result.end() - result.start() + 1;
      let expression = &item[index..index + len - 1];
      index += len;
      let mut define = false;
      if index < item.len() - 1 {
        let result = rg_def.find(&item[index..]);
        if result.is_some() {
          define = true;
          index += result.unwrap().end() + 1;
        }
      }
      let mut result = None;
      if index < item.len() - 1 {
        result = rg_action.find(&item[index..]);
      }
      let rule = LexRuleIntl::new(expression,
        match result {
          Some(action) => Some(Function::new_with_args("get, set, set_name, set_name_from_hash, pass", action.as_str())),
          None => None
        }, define, rules.len());
      rules.insert(name, rule);
    }

    // Converts names to regular expressions
    let rg_name_expression = Regex::new(r"\{(_|[A-Za-z])(_|[A-Za-z]|[0-9])*\}").unwrap();
    loop {
      let mut expressions: HashMap<usize, String> = HashMap::new();
      for (name, rule) in &rules {
        loop {
          let expression = (match expressions.get(name) {
            Some(expr) => expr,
            _ => &rule.expression
          }).clone();
          let result = rg_name_expression.find(&expression);
          if result == None {break};
          let result = result.unwrap();
          let name1 = &expression[result.start()+1..result.end()-1];
          let name1 = hash(&(name1.to_string()));
          let rule1 = rules.get(&name1);
          if rule1.is_some() {
            let rule1 = rule1.unwrap();
            expressions.insert(*name, expression.
              replace(&expression[result.start()..result.end()], &rule1.expression));
          }
        }
      }
      if expressions.len() == 0 {break};
      for (name, expression) in &expressions {
        let old_rule = rules.get(name).unwrap();
        let new_rule = LexRuleIntl::new(expression, old_rule.action.clone(), old_rule.define, old_rule.position);
        rules.insert(*name, new_rule);
      }
    }
    // creates regular expressions
    for (name, rule) in rules {
      if rule.define {continue};
      self.rules.push(LexRule {
        name,
        expression: Regex::new(&rule.expression).unwrap(),
        action: rule.action.clone(),
        define: rule.define,
        position: rule.position
      });
    }
    //sort rules
    self.rules.sort_by_key(|rule| rule.position);

    self.reg_exp = value;
  }

  pub fn error(&self) -> bool {
    self.error
  }
}

#[wasm_bindgen(module = "/src/lex.js")]
extern "C" {
  #[wasm_bindgen(catch)]
  pub fn do_lex_action(action: &Function, context: &JsValue,
    get: &dyn Fn() -> Vec<u8>, set: &mut dyn FnMut(Vec<u8>), set_name: &mut dyn FnMut(String), set_name_from_hash: &mut dyn FnMut(usize), pass: &mut dyn FnMut()
  ) -> Result<(), JsValue>;
}