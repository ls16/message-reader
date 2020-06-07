use std::collections::{HashMap, BTreeMap, HashSet};
use std::rc::Rc;
use js_sys::Function;
use wasm_bindgen::prelude::JsValue;

use super::utils::*;
use super::lex::*;
use super::parser::*;
use super::dfa_grammar::*;

#[derive(Debug, Clone)]
pub struct State {
  accept: usize,
  action: Option<Rc<Function>>
}

impl State {
  fn new(accept: usize, action: Option<Rc<Function>>) -> Self {
    Self {
      accept,
      action
    }
  }

  pub fn accept(&self) -> usize {
    self.accept
  }

  pub fn action(&self) -> &Option<Rc<Function>> {
    &self.action
  }
}

#[derive(Debug)]
struct GotoStates1 {
  states: HashMap<usize, usize, BuildUsizeHasher>
}

impl GotoStates1 {
  pub fn new() -> Self {
    Self {
      states: HashMap::default()
    }
  }

  pub fn state(&self, state: usize) -> Option<&usize> {
    self.states.get(&state)
  }

  pub fn set_state(&mut self, state: usize, goto_state: usize) {
    self.states.insert(state, goto_state);
  }
}

#[derive(Debug)]
pub struct GotoStates1Opt {
  states: Vec<Option<usize>>
}

impl GotoStates1Opt {
  fn from(states: &GotoStates1) -> Self {
    let mut len = *(states.states.keys().max().unwrap_or(&0));
    if len > 0 {len += 1}
    if states.states.len() > len {
      len = states.states.len();
    }
    let mut states_opt: Vec<Option<usize>> = Vec::with_capacity(len);
    for _ in 0..len {
      states_opt.push(None);
    }

    for key in states.states.keys() {
      states_opt[*key] = Some(*states.states.get(key).unwrap());
    }
    Self {
      states: states_opt
    }
  }

  pub fn state(&self, state: usize) -> Option<&usize> {
    match self.states.get(state) {
      Some(st) => {
        match st {
          Some(st) => Some(st),
          _ => None
        }
      },
      _ => None
    }
  }
}

#[derive(Debug)]
struct GotoStates {
  states: HashMap<usize, GotoStates1, BuildUsizeHasher>
}

impl GotoStates {
  pub fn new() -> Self {
    Self {
      states: HashMap::default()
    }
  }

  pub fn state(&self, state: usize, code: usize) -> Option<&usize> {
    let state = self.states.get(&state);
    match state {
      Some(st) => st.state(code),
      None => None
    }
  }

  pub fn set_state(&mut self, state: usize, code: usize, goto_state: usize) {
    let mut state1 = self.states.get_mut(&state);
    if state1.is_none() {
      self.states.insert(state, GotoStates1::new());
      state1 = self.states.get_mut(&state);
    }
    let state1 = state1.unwrap();
    state1.set_state(code, goto_state);
  }

  #[allow(dead_code)]
  pub fn state_exists(&self, state: usize) -> bool {
    self.states.get(&state).is_some()
  }
}

#[derive(Debug)]
pub struct GotoStatesOpt {
  states: Vec<Option<GotoStates1Opt>>
}

impl GotoStatesOpt {
  fn from(states: &GotoStates) -> Self {
    let mut len = *(states.states.keys().max().unwrap_or(&0));
    if len > 0 {len += 1}
    if states.states.len() > len {
      len = states.states.len();
    }
    let mut states_opt: Vec<Option<GotoStates1Opt>> = Vec::with_capacity(len);
    for _ in 0..len {
      states_opt.push(None);
    }

    for key in states.states.keys() {
      states_opt[*key] = match states.states.get(key) {
        Some(state) => Some(GotoStates1Opt::from(state)),
        _ => None
      };
    }
    Self {
      states: states_opt
    }
  }

  pub fn state(&self, state: usize, code: usize) -> Option<&usize> {
    match self.states.get(state) {
      Some(st) => match st {
        Some(st) => st.state(code),
        _ => None
      },
      None => None
    }
  }

  pub fn state_exists(&self, state: usize) -> bool {
    match self.states.get(state) {
      Some(st) => st.is_some(),
      _ => false
    }
  }

}

pub struct DFABuilder {
}

impl DFABuilder {
  pub fn new() -> Self {
    Self {
    }
  }

  fn parse(parser: &mut Parser, text: String, exec_context: ExecContext) -> Result<ExecContext, JsValue> {
    let id = 0;
    let js_context = cast_exec_context_to_js_value(id, exec_context);
    parser.set_text(text);
    let result = parser.parse(&js_context)?;
    if result == ParseResult::ParseWait {
      return Err(JsValue::from("Error parse"));
    }
    let exec_context = return_exec_context(id);
    Ok(exec_context)
  }

  fn build_ast(re_def: String) -> Result<ExecContext, JsValue> {
    let mut dfa_lex = Box::new(Lex::new("".to_string()));
    dfa_lex.set_regular_definition_text(reg_exp().to_string());
    let mut dfa_parser = Parser::new(dfa_lex);
    dfa_parser.disable_state_logging();
    let _ = dfa_parser.set_grammar(grammar().to_string());

    let mut lex = Lex::new("".to_string());
    lex.set_regular_definition_text(re_def);

    let mut v_context: Vec<ExecContext> = vec!(ExecContext::new());
    for i in 0..lex.rules().len() {
      let rule = &lex.rules()[i];
      let exec_context = v_context.pop().unwrap();
      let prev_root_id = exec_context.last_item_id();
      let mut exec_context = DFABuilder::parse(&mut dfa_parser, rule.expression().as_str().to_string(), exec_context)?;
      let builder = exec_context.builder();
      let mut attrs = Attributes::new();
      attrs.attrs.insert(hash("accept"), Box::new(rule.name()));
      if let Some(action) = rule.action() {
        attrs.attrs.insert(hash("action"), Box::new(action.clone()));
      }
      let leaf = builder.add_leaf("#".to_string(), vec!(), Some(attrs)).unwrap();
      let last = builder.last().unwrap();
      let _ = builder.add_node(".".to_string(), last, Some(leaf), None);

      match prev_root_id {
        Some(prev_root_id) => {
          let last = builder.last().unwrap();
          let prev_root = builder.by_id(prev_root_id).unwrap();
          let _ = builder.add_node("|".to_string(), prev_root, Some(last), None);
        },
        _ => {}
      }
      v_context.push(exec_context);
    }
    Ok(v_context.pop().unwrap())
  }

  fn nullable(&self, item: &Rc<dyn ASTItem>) -> bool {
    match item.type_item() {
      ASTItemType::Leaf => {
        return item.value().len() == 0;
      },
      ASTItemType::Node => {
        match item.name() {
          "|" => {
            return self.nullable(&item.left().unwrap()) ||
              match item.right() {
                Some(ref right) => self.nullable(right),
                _ => true
              }
          },
          "." => {
            return self.nullable(&item.left().unwrap()) &&
              match item.right() {
                Some(ref right) => self.nullable(right),
                _ => true
              }
        },
          "*" => {return true},
          _ => {return true}
        }
      }
    }
  }

  fn first<'a>(&'a self, item: &Rc<dyn ASTItem>, items: &BTreeMap<usize, Rc<dyn ASTItem>>, first_items: &'a mut HashMap<usize, HashSet<usize>>) -> &mut HashSet<usize> {
    let item_id = item.id();

    if !first_items.contains_key(&item_id) {
      let mut result: HashSet<usize> = HashSet::new();
      let item = items.get(&item_id).expect(format!("Unknown item id: {}", item_id).as_str());
      match item.type_item() {
        ASTItemType::Leaf => {
          if item.value().len() != 0 || item.name() == "#" {
            result.insert(item.id());
          }
        },
        ASTItemType::Node => {
          let left = item.left().unwrap();
          match item.name() {
            "|" => {
              for value in self.first(&left, items, first_items).iter() {
                result.insert(*value);
              }
              let right = item.right().unwrap();
              for value in self.first(&right, items, first_items).iter() {
                result.insert(*value);
              }
            },
            "." => {
              if self.nullable(&left) {
                for value in self.first(&left, items, first_items).iter() {
                  result.insert(*value);
                }
                let right = item.right().unwrap();
                for value in self.first(&right, items, first_items).iter() {
                  result.insert(*value);
                }
              } else {
                for value in self.first(&left, items, first_items).iter() {
                  result.insert(*value);
                }
              }
            }
            _ => {
              for value in self.first(&left, items, first_items).iter() {
                result.insert(*value);
              }
            }
          }
        }
      }

      first_items.insert(item_id, result);
    }

    first_items.get_mut(&item_id).unwrap()
  }

  fn last<'a>(&'a self, item: &Rc<dyn ASTItem>, items: &BTreeMap<usize, Rc<dyn ASTItem>>, last_items: &'a mut HashMap<usize, HashSet<usize>>) -> &mut HashSet<usize> {
    let item_id = item.id();

    if !last_items.contains_key(&item_id) {
      let mut result: HashSet<usize> = HashSet::new();
      let item = items.get(&item_id).expect(format!("Unknown item id: {}", item_id).as_str());
      match item.type_item() {
        ASTItemType::Leaf => {
          if item.value().len() != 0 || item.name() == "#" {
            result.insert(item.id());
          }
        },
        ASTItemType::Node => {
          let left = item.left().unwrap();
          match item.name() {
            "|" => {
              for value in self.last(&left, items, last_items).iter() {
                result.insert(*value);
              }
              let right = item.right().unwrap();
              for value in self.last(&right, items, last_items).iter() {
                result.insert(*value);
              }
            },
            "." => {
              let right = item.right().unwrap();
              if self.nullable(&right) {
                for value in self.last(&right, items, last_items).iter() {
                  result.insert(*value);
                }
                for value in self.last(&left, items, last_items).iter() {
                  result.insert(*value);
                }
              } else {
                for value in self.last(&right, items, last_items).iter() {
                  result.insert(*value);
                }
              }
            },
            _ => {
              for value in self.last(&left, items, last_items).iter() {
                result.insert(*value);
              }
            }
          }
        }
      }

      last_items.insert(item_id, result);
    }

    last_items.get_mut(&item_id).unwrap()
  }

  fn follow<'a>(&'a self, item: &Rc<dyn ASTItem>, items: &BTreeMap<usize, Rc<dyn ASTItem>>,
    first_items: &'a mut HashMap<usize, HashSet<usize>>,
    last_items: &'a mut HashMap<usize, HashSet<usize>>,
    follow_items: &'a mut HashMap<usize, HashSet<usize>>) -> &HashSet<usize> {
    let item_id = item.id();

    if !follow_items.contains_key(&item_id) {
      let mut result: HashSet<usize> = HashSet::new();

      for (_, item) in items.iter() {
        match item.name() {
          "." => {
            let left = item.left().unwrap();
            if self.last(&left, items, last_items).contains(&item_id) {
              let right = item.right().unwrap();
              for value in self.first(&right, items, first_items).iter() {
                result.insert(*value);
              }
            }
          },
          "*" => {
            if self.last(&item, items, last_items).contains(&item_id) {
              let left = item.left().unwrap();
              for value in self.first(&left, items, first_items).iter() {
                result.insert(*value);
              }
            }
          },
          _ => {}
        }
      }

      follow_items.insert(item_id, result);
    }

    follow_items.get(&item_id).unwrap()
  }

  fn get_codes(items: &BTreeMap<usize, Rc<dyn ASTItem>>) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();
    let mut added: HashSet<u8> = HashSet::new();
    for (_, item) in items {
      if item.name() == "code" && item.value().len() > 0 {
        let code = item.value()[0];
        if added.contains(&code) {continue}
        added.insert(code);
        result.push(code);
      }
    }
   result
  }
}

pub fn build(re_def: String) -> (Vec<Option<State>>, GotoStatesOpt) {
  let exec_context = DFABuilder::build_ast(re_def);
  let mut exec_context = exec_context.unwrap();
  let ast_builder = exec_context.builder();
  let items = ast_builder.items();
  let dfa_builder = DFABuilder::new();
  let mut first_items: HashMap<usize, HashSet<usize>> = HashMap::new();
  let mut last_items: HashMap<usize, HashSet<usize>> = HashMap::new();
  let mut follow_items: HashMap<usize, HashSet<usize>> = HashMap::new();

  let mut s: Vec<HashSet<usize>> = vec!();
  s.insert(0, dfa_builder.first(&ast_builder.last().unwrap(), &items, &mut first_items).clone());
  let codes = DFABuilder::get_codes(&items);
  let mut marked: BTreeMap<usize, bool> = BTreeMap::new();
  marked.insert(0, false);
  let mut states: HashMap<usize, State> = HashMap::new();
  let mut goto_states = GotoStates::new();
  loop {
    let index = marked.values().position(|&value| value == false);
    if index == None {break}

    let index = index.unwrap();
    marked.insert(index, true);
    for code in &codes {
      let mut u: HashSet<usize> = HashSet::new();
      for p in s[index].iter() {
        let item = items.get(p).unwrap();
        if item.name() == "code" && item.value().len() > 0 && item.value()[0] == *code {
          for val in dfa_builder.follow(item, &items, &mut first_items, &mut last_items, &mut follow_items).iter() {
            u.insert(*val);
          };
        }
      }
    if u.len() == 0 {continue}
    let mut u_index = s.iter().position(|s_item| {
      u.len() == s_item.len() && u.iter().all(|&val| s_item.contains(&(val)))
    });
    if u_index == None {
        s.push(u);
        let u_index1 = s.len() - 1;
        marked.insert(u_index1, false);
        u_index = Some(u_index1);
      }
      goto_states.set_state(index, *code as usize, u_index.unwrap());
    }
  }

  for index in 0..s.len() {
    let s_item = &s[index];
    s_item.iter().any(|p| {
      let item = items.get(p).unwrap();
      if item.name() == "#" {
        let mut state = State::new(item.attr(&hash("accept")).unwrap().as_usize().unwrap(), None);
        state.action = match item.attr(&hash("action")) {
          Some(ref attr) => match attr.as_function() {
            Some(func) => {
              Some(Rc::new(func.clone()))
            },
            _ => None
          },
          None => None
        };
        states.insert(index, state);
        true
      } else {false}
    });
  }

  // remove dead-end transitions
  for (state, states1) in &goto_states.states {
    let mut dead = true;
    for code in states1.states.keys() {
      match goto_states.state(*state, *code) {
        Some(goto_state) => {
          if goto_state != state {
            dead = false;
            break;
          }
        },
        _ => {}
      }
    }
    if dead {
      match states.get(state) {
        Some(_) => {dead = false}
        _ => {}
      }
    }
    if dead {
      states.remove(state);
    }
  }

  //convert states map to vec
  let mut len = *(states.keys().max().unwrap_or(&0));
  if len > 0 {len += 1}
  if states.len() > len {
    len = states.len();
  }
  let mut states_opt: Vec<Option<State>> = Vec::with_capacity(len);
  for _ in 0..len {
    states_opt.push(None);
  }

  for key in states.keys() {
    states_opt[*key] = match states.get(key) {
      Some(state) => Some(state.clone()),
      _ => None
    };
  }

  //convert goto states map to vec
  let goto_states_opt = GotoStatesOpt::from(&goto_states);

  (states_opt, goto_states_opt)
}