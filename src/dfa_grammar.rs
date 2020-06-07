use std::char;
use std::rc::Rc;
use std::fmt::Debug;
use js_sys::Array;
use wasm_bindgen::prelude::{JsValue, wasm_bindgen};

use super::utils::*;

pub fn reg_exp() -> &'static str {
  "
   '+' \\+
   '-' \\-
   '*' \\*
   '|' \\|
   '?' \\?
   ',' ,
   '^' \\^
   '(' \\(
   ')' \\)
   '[' \\[
   ']' \\]
   '{' \\{
   '}' \\}
   char \\S {let tkn = this.to_codes(get(), 'utf8'); set_name(tkn[0]); set(tkn[1])}
   escape \\\\{char} {let tkn = this.to_codes(get().slice(1), 'utf8'); set_name(tkn[0]); set(tkn[1])}
   escape_hex \\\\x[0-9A-Fa-f][0-9A-Fa-f] {let tkn = this.to_codes(get().slice(2), 'hex'); set_name(tkn[0]); set(tkn[1])}
   escape_unicode \\\\u[0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f] {let tkn = this.to_codes(get().slice(2), 'unicode'); set_name(tkn[0]); set(tkn[1])}
  "
}

pub fn grammar() -> &'static str {
  "
  start: pattern;
  pattern: disjunction {bind(id(0))};

  disjunction: alternative '|' disjunction {bind(this.add_node('|', id(2), id(0)))};
  disjunction: alternative {bind(id(0))};

  alternative: alternative term {bind(this.add_node('.', id(1), id(0)))};
  alternative: {bind(this.add_leaf('code', ''))};

  term: atom '{' digits ',' digits '}' {bind(this.build_tree_to_duplicates2(id(5), get(3), get(1)))};
  term: atom '{' digits ',' '}' {bind(this.build_tree_to_duplicates2(id(4), get(2), [48] /* 0 */))};
  term: atom '{' digits '}' {bind(this.build_tree_to_duplicates1(id(3), get(1)))};
  term: atom '?' {bind(this.build_tree_to_duplicates3(id(1), 0, 1))};
  term: atom '+' {bind(this.build_tree_to_duplicates3(id(1), 1, 0))};
  term: atom '*' {bind(this.build_tree_to_duplicates3(id(1), 0, 0))};
  term: atom {bind(id(0))};
  digits: 'codes' {bind(this.build_tree_to_codes(get(0)))};
  digits: 'code' {set(0); bind(this.add_leaf('code', get(0)))};

  atom: character_class {bind(id(0))};
  atom: '(' pattern ')' {bind(id(1))};
  atom: 'codes' {bind(this.build_tree_to_codes(get(0)))};
  atom: 'code' {bind(this.add_leaf('code', get(0)))};

  character_class: '[' '^' class_ranges_nonmatch ']' {bind(id(1))};
  character_class: '[' class_ranges_match ']' {bind(id(1))};

  class_ranges_nonmatch: nonempty_class_ranges_nonmatch {bind(id(0))};
  class_ranges_nonmatch: {bind(this.add_leaf('code', ''))};
  nonempty_class_ranges_nonmatch: class_atom_nonmatch '-' class_atom_nonmatch class_ranges_nonmatch {let node_type = get(0)[0] == 0 ? '.' : '|'; bind(this.add_node(node_type, this.build_tree_to_range(id(3), get(3), id(1), get(1), true), id(0)))};
  nonempty_class_ranges_nonmatch: class_atom_nonmatch {bind(id(0))};
  class_atom_nonmatch: 'code' {set(0); bind(this.add_leaf('code_not', get(0)))};

  class_ranges_match: nonempty_class_ranges_match {set_val([1]); bind(id(0))};
  class_ranges_match: {set_val([0]); bind(this.add_leaf('code', ''))};
  nonempty_class_ranges_match: class_atom_match {bind(id(0))};
  nonempty_class_ranges_match: class_atom_match '-' class_atom_match class_ranges_match {let node_type = get(0)[0] == 0 ? '.' : '|'; bind(this.add_node(node_type, this.build_tree_to_range(id(3), get(3), id(1), get(1), false), id(0)))};
  class_atom_match: 'code' {set(0); bind(this.add_leaf('code', get(0)))};
  "
}

#[wasm_bindgen(module = "/src/dfa_grammar.js")]
extern "C" {
  pub fn cast_exec_context_to_js_value(id: usize, ec: ExecContext) -> JsValue;
  pub fn return_exec_context(id: usize) -> ExecContext;
}

#[derive(Debug)]
struct TokenJS (String, Vec<u8>);

impl TokenJS {
  ///Gets value as String
  pub fn value_to_string(&self) -> String {
    String::from_utf8(self.1.clone()).unwrap_or_default()
  }
}


#[wasm_bindgen]
#[derive(Clone)]
pub struct ExecContext {
  builder: ASTBuilder
}

impl ParserExecContext for ExecContext {
  fn builder(&mut self) -> &mut ASTBuilder {
    &mut self.builder
  }
}

impl ExecContext {
  pub fn last_item_id(&self) -> Option<usize> {
    match self.builder.last() {
      Some(ref last) => Some(last.id()),
      _ => None
    }
  }

}

#[wasm_bindgen]
impl ExecContext {
  pub fn new() -> Self {
    ExecContext {
      builder: ASTBuilder::new()
    }
  }

  pub fn to_codes(&self, tkn_value: Vec<u8>, encoding: &str) -> Result<JsValue, JsValue> {
    fn to_js_token(token: &TokenJS) -> Result<JsValue, JsValue> {
      let arr = Array::new();
      let name = JsValue::from(&token.0);
      let value = Array::new();
      for v in token.1.iter() {
        value.push(&JsValue::from(*v));
      }
      arr.push(&name);
      arr.push(&value);
      Ok(JsValue::from(arr))
    }

    fn convert(token: TokenJS, encoding: &str, name: Option<String>) -> Result<JsValue, JsValue> {
      let name = match name {
        Some(name) => name,
        _ => "codes".to_string()
      };
      let token = match encoding {
        "utf8" => {
          TokenJS(name, token.1.to_vec())
        },
        _ => {
          panic!("Unknown encoding: '{}'", encoding);
        }
      };

      to_js_token(&token)
    }

    let mut token = TokenJS("".to_string(), tkn_value);
    match encoding {
      "utf8" => {
        if token.1.len() == 1 && token.1[0] < 128u8 {
          token.0 = "code".to_string();
          to_js_token(&token)
        } else {
          convert(token, encoding, None)
        }
      },
      "unicode" => {
        if token.1.len() != 4 {return Err(JsValue::from("The length of 'unicode' value must be equal 4 bytes"))}
        let str_val = token.value_to_string();
        let code = u32::from_str_radix(&str_val, 16).expect(format!("Bad hex format number: {:?}", &str_val).as_str());
        let ch = char::from_u32(code).expect(format!("Bad number for unicode: {}", code).as_str());
        to_js_token(&TokenJS("codes".to_string(), ch.to_string().as_bytes().to_vec()))
      },
      "hex" => {
        if token.1.len() != 2 {return Err(JsValue::from("The length of 'hex' value must be equal 2 bytes"))}
        let str_val = token.value_to_string();
        let data = u8::from_str_radix(&str_val, 16).expect(format!("Bad hex format number: {:?}", &str_val).as_str());
        to_js_token(&TokenJS("code".to_string(), data.to_be_bytes().to_vec()))
      },
      _ => {Err(JsValue::from(format!("Error encoding format {:?}", encoding)))}
    }
  }

  fn to_digit(&self, value: Vec<u8>) -> Result<usize, JsValue> {
    let result = String::from_utf8(value);
    if result.is_err() {return Err(JsValue::from(format!("Error convert to digit: {:?}", result).as_str()));}
    let result = result.unwrap().parse();
    if result.is_err() {return Err(JsValue::from(format!("Error convert to digit: {:?}", result).as_str()));}
    Ok(result.unwrap())
  }

  fn clone(&mut self, item: Rc<dyn ASTItem>) -> Result<Rc<dyn ASTItem>, JsValue> {
    if item.type_item() == ASTItemType::Node {
      let left = self.clone(item.left().unwrap())?;
      let right = match item.right() {
        Some(data) => {
          let data = self.clone(data)?;
          Some(data)
        },
        _ => None
      };
      self.builder.add_node(item.name().to_string(), left, right, None)
    } else {
      self.builder.add_leaf(item.name().to_string(), item.value().clone(), None)
    }
  }

  fn err(&self) -> Result<Rc<dyn ASTItem>, JsValue> {
    Err(JsValue::from("'atom' not found in tree."))
  }

  fn v1(&mut self, atom_id: usize, min: usize) -> Result<Rc<dyn ASTItem>, JsValue> {
    let count = min;
    if count == 0 {return Err(JsValue::from("NULL"));}
    let atom_node = self.builder.by_id(atom_id);
    if atom_node.is_none() {return self.err()}
    let atom_node = atom_node.unwrap();
    let mut node = atom_node.clone();
    for _i in 1..count {
      let right = Some(self.clone(atom_node.clone())?);
      node = self.builder.add_node(".".to_string(), node, right, None)?;
    }
    Ok(node)
  }

  fn v2(&mut self, atom_id: usize, min: usize) -> Result<Rc<dyn ASTItem>, JsValue> {
    let atom_node = self.builder.by_id(atom_id);
    if atom_node.is_none() {return self.err()}
    let atom_node = atom_node.unwrap();
    let left = self.clone(atom_node.clone())?;
    let mut node = self.builder.add_node("*".to_string(), left, None, None)?;
    if min > 0 {
      let left = self.v1(atom_id, min)?;
      node = self.builder.add_node(".".to_string(), left, Some(node), None)?;
    }
    Ok(node)
  }

  fn v3(&mut self, atom_id: usize, min: usize, max: usize) -> Result<Rc<dyn ASTItem>, JsValue> {
    if min > max {
      return Err(JsValue::from(format!("min number:{} less than max number:{}", min, max)));
    }
    let atom_node = self.builder.by_id(atom_id);
    if atom_node.is_none() {return self.err()}
    let atom_node = atom_node.unwrap();
    let mut node;
    if min == 0 {
      node = self.builder.add_leaf("code".to_string(), vec![], None)?;
    } else {
      node = self.v1(atom_id, min)?;
    }
    for _i in min..max {
      let left1 = self.clone(atom_node.clone())?;
      let right1 = self.builder.add_leaf("code".to_string(), vec![], None)?;
      let right2 = self.builder.add_node("|".to_string(), left1, Some(right1), None)?;
      node = self.builder.add_node(".".to_string(), node, Some(right2), None)?;
    }
    Ok(node)
  }

  pub fn build_tree_to_duplicates1(&mut self, atom_id: usize,
    min_codes: Vec<u8>) -> Result<usize, JsValue> {
    self.build_tree_to_duplicates3(atom_id, self.to_digit(min_codes)?, None)
  }

  pub fn build_tree_to_duplicates2(&mut self, atom_id: usize,
    min_codes: Vec<u8>, max_codes: Vec<u8>) -> Result<usize, JsValue> {
    self.build_tree_to_duplicates3(atom_id, self.to_digit(min_codes)?, Some(self.to_digit(max_codes)?))
  }

  pub fn build_tree_to_duplicates3(&mut self, atom_id: usize,
    min: usize, max: Option<usize>) -> Result<usize, JsValue> {
    let result = match max {
      None => {
        if min > 0 {
          self.v1(atom_id, min)?
        } else {
          self.v2(atom_id, min)?
        }
      },
      Some(0) => self.v2(atom_id, min)?,
      Some(max) => self.v3(atom_id, min, max)?
    };
    Ok(result.id())
  }

  pub fn build_tree_to_codes(&mut self, codes: Vec<u8>) -> Result<usize, JsValue> {
    if codes.len() < 2 {return Err(JsValue::from("The length of 'codes' must not be less than 2"))}
    let mut index = 0;
    let leaf = self.builder.add_leaf("code".to_string(), vec!(codes[index]), None)?;
    index += 1;
    let right = self.builder.add_leaf("code".to_string(), vec!(codes[index]), None)?;
    index += 1;
    let _ = self.builder.add_node(".".to_string(), leaf, Some(right), None);
    for _i in index..codes.len() {
      let left = self.builder.last().unwrap().clone();
      let right = self.builder.add_leaf("code".to_string(), vec!(codes[_i]), None)?;
      let _ = self.builder.add_node(".".to_string(), left, Some(right), None);

    }
    Ok(self.builder.last().unwrap().id())
  }

  pub fn build_tree_to_range(&mut self, start_id: usize, start_value: Vec<u8>, end_id: usize, end_value: Vec<u8>, not_items: Option<bool>) -> Result<usize, JsValue> {
    if start_value.len() > 1 {
      return Err(JsValue::from(format!("start value {:?} takes more than one byte", start_value)));
    }
    if end_value.len() > 1 {
      return Err(JsValue::from(format!("end value {:?} takes more than one byte", end_value)));
    }
    let start_code = start_value[0];
    let end_code = end_value[0];
    if start_code > end_code {
      return Err(JsValue::from(format!("The starting value: {:?} must not be greater than the final value: {:?}.", start_code, end_code)));
    }
    let mut code = start_code;
    let name = match not_items.unwrap_or(false) {
      true => "code_not".to_string(),
      false => "code".to_string()
    };
    let right_id = match end_code > start_code {
      true => {
        let leaf = self.builder.add_leaf(name.clone(), vec!(code), None)?;
        code += 1;
        Some(leaf.id())
      },
      false => None
    };
    match self.builder.last() {
      Some(ref _last) => {
        let _ = self.builder.add_node_id("|".to_string(), start_id, right_id, None)?;
      },
      _ => {}
    }
    loop {
      if code > end_code {break}
      let right_id = match code < end_code {
        true => self.builder.add_leaf_id(name.clone(), vec!(code), None)?,
        false => end_id
      };
      let left_id = match self.builder.last() {
        Some(ref last) => last.id(),
        _ => start_id
      };
      let _ = self.builder.add_node_id("|".to_string(), left_id, Some(right_id), None)?;
      if code == 255 {break}
      code += 1;
    }
    Ok(self.builder.last().unwrap().id())
  }

  pub fn add_leaf(&mut self, name: String, value: Vec<u8>) -> Result<usize, JsValue> {
    self.builder.add_leaf_id(name, value, None)
  }

  pub fn add_node(&mut self, name: String, left_id: usize, right_id: Option<usize>) -> Result<usize, JsValue> {
    self.builder.add_node_id(name, left_id, right_id, None)
  }

}
