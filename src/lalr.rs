use std::collections::HashMap;
use std::rc::Rc;
use js_sys::Function;
use wasm_bindgen::prelude::JsValue;

use super::utils::*;
use super::lex::*;

fn reg_exp() -> &'static str {
  "space                \\s+
    digit               [0-9]       DEF
    letter              _|[A-Za-z]  DEF
    nonterm_name        {letter}({letter}|{digit})*
    term_name           \'\\S+\'    {set(get().slice(1,-1))}
    colon               :
    vert_line           \\|
    rust_action_code    \\[[\\s|\\S]+?\\]
    action_code         \\{[\\s|\\S]+?\\}
    semicolon           ;
    error               error
  "
}

fn rust_action_regular_definition_text() -> &'static str {
  "space                \\s+        {pass()}
    digit               [0-9]       DEF
    set                 set
    index               {digit}({digit}|{digit})*
    '('                 \\(
    ','                 ,
    ')'                 \\)
  "
}

#[derive(Debug, Clone)]
struct GotoStates1 {
  states: HashMap<usize, usize, BuildUsizeHasher>
}

impl GotoStates1 {
  pub fn new() -> Self {
    Self {
      states: HashMap::default()
    }
  }

  #[allow(dead_code)]
  pub fn state(&self, name: &usize) -> Option<&usize> {
    self.states.get(name)
  }

  pub fn set_state(&mut self, name: usize, state: usize) {
    self.states.insert(name, state);
  }
}

#[derive(Debug)]
struct GotoStates1Opt {
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

  #[allow(dead_code)]
  pub fn state(&self, state: &usize, name: &usize) -> Option<&usize> {
    let state = self.states.get(state);
    match state {
      Some(st) => st.state(name),
      None => None
    }
  }

  pub fn set_state(&mut self, state: usize, name: usize, goto_state: usize) {
    let mut state1 = self.states.get_mut(&state);
    if state1.is_none() {
      self.states.insert(state, GotoStates1::new());
      state1 = self.states.get_mut(&state);
    }
    let state1 = state1.unwrap();
    state1.set_state(name, goto_state);
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

  pub fn new() -> Self {
    Self {
      states: vec!()
    }
  }

  pub fn state(&self, state: usize, name: usize) -> Option<&usize> {
    let state = self.states.get(state);
    match state {
      Some(st) => match st {
        Some(st) => st.state(name),
        _ => None
      },
      None => None
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActionState {
  Shift,
  Reduce,
  Accept
}

#[derive(Debug, Clone)]
pub struct ActionState2 {
  state: ActionState,
  goto: Option<usize>,
  production: Option<Rc<GrammarProduction>>
}

impl ActionState2 {
  pub fn new(state: ActionState, goto: Option<usize>, production: Option<Rc<GrammarProduction>>) -> Self {
    Self {
      state,
      goto,
      production
    }
  }

  pub fn state(&self) -> &ActionState {
    &self.state
  }

  pub fn goto(&self) -> &Option<usize> {
    &self.goto
  }

  pub fn production(&self) -> &Option<Rc<GrammarProduction>> {
    &self.production
  }

  pub fn production_name(&self) -> Option<usize> {
    match &self.production {
      Some(data) => Some(data.name()),
      _ => None
    }
  }

  pub fn production_len(&self) -> Option<usize> {
    match &self.production {
      Some(data) => Some(data.len()),
      _ => None
    }
  }
}

#[derive(Debug, Clone)]
struct ActionStates1 {
  states: HashMap<usize, ActionState2, BuildUsizeHasher>
}

impl ActionStates1 {
  pub fn new() -> Self {
    Self {
      states: HashMap::default()
    }
  }

  #[allow(dead_code)]
  pub fn state(&self, name: &usize) -> Option<&ActionState2> {
    self.states.get(name)
  }

  pub fn set_state(&mut self, name: usize, state: ActionState2) {
    self.states.insert(name, state);
  }
}

#[derive(Debug)]
struct ActionStates1Opt {
  states: Vec<Option<ActionState2>>
}

impl ActionStates1Opt {
  fn from(states: &ActionStates1) -> Self {
    let mut len = *(states.states.keys().max().unwrap_or(&0));
    if len > 0 {len += 1}
    if states.states.len() > len {
      len = states.states.len();
    }
    let mut states_opt: Vec<Option<ActionState2>> = Vec::with_capacity(len);
    for _ in 0..len {
      states_opt.push(None);
    }

    for key in states.states.keys() {
      states_opt[*key] = Some(states.states.get(key).unwrap().clone());
    }
    Self {
      states: states_opt
    }
  }

  pub fn state(&self, name: usize) -> Option<&ActionState2> {
    match self.states.get(name) {
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
struct ActionStates {
  states: HashMap<usize, ActionStates1, BuildUsizeHasher>
}

impl ActionStates {
  pub fn new() -> Self {
    Self {
      states: HashMap::default()
    }
  }

  #[allow(dead_code)]
  pub fn state(&self, state: &usize, name: &usize) -> Option<&ActionState2> {
    let state = self.states.get(state);
    match state {
      Some(st) => st.state(name),
      None => None
    }
  }

  #[allow(dead_code)]
  pub fn cloned_state(&self, state: &usize, name: &usize) -> Option<ActionState2> {
    let state = self.states.get(state);
    match state {
      Some(st) => {
        match st.state(name) {
          Some(st) => Some((*st).clone()),
          None => None
        }
      },
      None => None
    }
  }

  pub fn set_state(&mut self, state: usize, name: usize, action_state: ActionState2) {
    let mut state1 = self.states.get_mut(&state);
    if state1.is_none() {
      self.states.insert(state, ActionStates1::new());
      state1 = self.states.get_mut(&state);
    }
    let state1 = state1.unwrap();
    state1.set_state(name, action_state);
  }

}

#[derive(Debug)]
pub struct ActionStatesOpt {
  states: Vec<Option<ActionStates1Opt>>
}

impl ActionStatesOpt {
  fn from(states: &ActionStates) -> Self {
    let mut len = *(states.states.keys().max().unwrap_or(&0));
    if len > 0 {len += 1}
    if states.states.len() > len {
      len = states.states.len();
    }
    let mut states_opt: Vec<Option<ActionStates1Opt>> = Vec::with_capacity(len);
    for _ in 0..len {
      states_opt.push(None);
    }

    for key in states.states.keys() {
      states_opt[*key] = match states.states.get(key) {
        Some(state) => Some(ActionStates1Opt::from(state)),
        _ => None
      };
    }
    Self {
      states: states_opt
    }
  }

  pub fn new() -> Self {
    Self {
      states: vec!()
    }
  }

  pub fn state(&self, state: usize, name: usize) -> Option<&ActionState2> {
    let state = self.states.get(state);
    match state {
      Some(st) => match st {
        Some(st) => st.state(name),
        _ => None
      },
      None => None
    }
  }

}

enum BuildState {
  WaitName,
  ProdName,
  WaitRight,
  Right,
  EndAction
}

pub struct GrammarBuilder {
}

impl GrammarBuilder {
  pub fn from_text(grammar: String) -> Result<Grammar, String> {
    let grammar = GrammarBuilder::build_grammar(grammar)?;
    Ok(grammar)
  }

  fn if_empty_error_production_add_e_term(production: &mut GrammarProduction) {
    if production.len() > 0 && production.symbol(0).name() == hash("error") {
      if production.len() == 1 {
        production.push_symbol(GrammarSymbol::e_term());
      }
    }
    if production.len() == 0 {
      production.push_symbol(GrammarSymbol::e_term());
    }
  }

  fn build_rust_action(rust_action_text: String) -> Result<RustAction, String> {

    fn get_token(lex: &mut Lex, ctx: &JsValue, err_message: &str) -> Result<Token, String> {
      let tkn = lex.get_token(ctx).expect("Error in get_token");
      if tkn == None {return Err(err_message.to_string())};
      Ok(tkn.unwrap())
    }

    fn get_index(lex: &mut Lex, ctx: &JsValue, err_message: &str) -> Result<usize, String> {
      let tkn = get_token(lex, ctx, err_message)?;
      if tkn.name() != hash("index") {return Err(err_message.to_string())};
      let str_val = tkn.value_to_string();
      let index = usize::from_str_radix(&str_val, 10);
      if index.is_err() {return Err(err_message.to_string())};
      Ok(index.unwrap())
    }

    fn get_action(lex: &mut Lex, ctx: &JsValue, err_message: &str,
      index: usize, index2: Option<usize>, index3: Option<usize>,
      index4: Option<usize>, index5: Option<usize>) -> Result<RustAction, String> {
        let tkn = lex.get_token(ctx).expect("Error in get_token");
        if tkn.is_some() {return Err(err_message.to_string())};
        Ok(RustAction::new(RustActionCommand::Set, index, index2, index3, index4, index5))
    }

    let len = rust_action_text.len();
    let rust_action_text = (&rust_action_text[1..len-1]).to_string();
    let null_context = &JsValue::NULL;
    let err_message = "Error parse rust action";
    let mut lex = Lex::new(rust_action_text);
    lex.set_regular_definition_text(rust_action_regular_definition_text().to_string());

    let tkn = get_token(&mut lex, null_context, err_message)?;
    if tkn.name() != hash("set") {return Err(err_message.to_string())};
    let tkn = get_token(&mut lex, null_context, err_message)?;
    if tkn.name() != hash("(") {return Err(err_message.to_string())};
    let index = get_index(&mut lex, null_context, err_message)?;
    let tkn = get_token(&mut lex, null_context, err_message)?;
    if tkn.name() == hash(")") {
      return get_action(&mut lex, null_context, err_message, index, None, None, None, None);
    };

    if tkn.name() != hash(",") {return Err(err_message.to_string())};
    let index2 = Some(get_index(&mut lex, null_context, err_message)?);
    let tkn = get_token(&mut lex, null_context, err_message)?;
    if tkn.name() == hash(")") {
      return get_action(&mut lex, null_context, err_message, index, index2, None, None, None);
    };

    if tkn.name() != hash(",") {return Err(err_message.to_string())};
    let index3 = Some(get_index(&mut lex, null_context, err_message)?);
    let tkn = get_token(&mut lex, null_context, err_message)?;
    if tkn.name() == hash(")") {
      return get_action(&mut lex, null_context, err_message, index, index2, index3, None, None);
    };

    if tkn.name() != hash(",") {return Err(err_message.to_string())};
    let index4 = Some(get_index(&mut lex, null_context, err_message)?);
    let tkn = get_token(&mut lex, null_context, err_message)?;
    if tkn.name() == hash(")") {
      return get_action(&mut lex, null_context, err_message, index, index2, index3, index4, None);
    };

    if tkn.name() != hash(",") {return Err(err_message.to_string())};
    let index5 = Some(get_index(&mut lex, null_context, err_message)?);
    let tkn = get_token(&mut lex, null_context, err_message)?;
    if tkn.name() == hash(")") {
      return get_action(&mut lex, null_context, err_message, index, index2, index3, index4, index5);
    };

    Err(err_message.to_string())
  }

  fn build_grammar(grammar: String) -> Result<Grammar, String> {
    let null_context = &JsValue::NULL;
    let mut lex = Lex::new(grammar);
    lex.set_regular_definition_text(reg_exp().to_string());
    let mut grammar = Grammar::new();
    let mut production_block: Vec<usize> = vec![];
    let mut production = GrammarProduction::new(hash(""), None);
    let mut prod_name: usize = 0;
    let mut state = BuildState::WaitName;
    loop {
      let tkn = lex.get_token(null_context).expect("Error in get_token");
      if tkn == None {break};
      let tkn = tkn.unwrap();
      if tkn.name() == hash("space") {continue};

      match &state {
        BuildState::WaitName => {
          if tkn.name() != hash("nonterm_name") {
            return Err("Expected name of production!".to_string());
          };
          prod_name = hash(&tkn.value_to_string());
          state = BuildState::ProdName;
        },
        BuildState::ProdName => {
          if tkn.name() != hash("colon") {
            return Err("Expected ':'!".to_string());
          };
          production = GrammarProduction::new(prod_name, None);
          production_block.clear();
          state = BuildState::WaitRight;
        },
        BuildState::WaitRight => {
          if tkn.name() == hash("nonterm_name") {
            production.push_symbol(GrammarSymbol::non_term(hash(&tkn.value_to_string()), None));
            state = BuildState::Right;
          } else if tkn.name() == hash("term_name") {
            production.push_symbol(GrammarSymbol::term(hash(&tkn.value_to_string()), None));
            state = BuildState::Right;
          } else if tkn.name() == hash("semicolon") {
            production.push_symbol(GrammarSymbol::e_term());
            grammar.push_production(Rc::new(production.clone()));
            production_block.push(grammar.len() - 1);
            state = BuildState::WaitName;
          } else if tkn.name() == hash("rust_action_code") {
            let action = GrammarBuilder::build_rust_action(tkn.value_to_string())?;
            production.add_attr(hash("rust_action"), Box::new(action.clone()));
            for i in &production_block {
              grammar.production_mut(*i).add_attr(hash("rust_action"), Box::new(action.clone()));
            }
            state = BuildState::EndAction;
          } else if tkn.name() == hash("action_code") {
            let func = Function::new_with_args("bind, id, lookup, get, set, set_val, set_name, set_name_from_hash, push_after", &tkn.value_to_string());
            production.add_attr(hash("action"), Box::new(func.clone()));
            for i in &production_block {
              grammar.production_mut(*i).add_attr(hash("action"), Box::new(func.clone()));
            }
            state = BuildState::EndAction;
          } else {
            return Err("Production symbol or action is expected!".to_string());
          }
        },
        BuildState::Right => {
          if tkn.name() == hash("nonterm_name") {
            production.push_symbol(GrammarSymbol::non_term(hash(&tkn.value_to_string()), None));
          } else if tkn.name() == hash("term_name") {
            production.push_symbol(GrammarSymbol::term(hash(&tkn.value_to_string()), None));
          } else if tkn.name() == hash("vert_line") {
            GrammarBuilder::if_empty_error_production_add_e_term(&mut production);
            grammar.push_production(Rc::new(production));
            production_block.push(grammar.len() - 1);
            production = GrammarProduction::new(prod_name.clone(), None);
          } else if tkn.name() == hash("rust_action_code") {
            let action = GrammarBuilder::build_rust_action(tkn.value_to_string())?;
            production.add_attr(hash("rust_action"), Box::new(action.clone()));
            for i in &production_block {
              grammar.production_mut(*i).add_attr(hash("rust_action"), Box::new(action.clone()));
            }
            state = BuildState::EndAction;
          } else if tkn.name() == hash("action_code") {
            let func = Function::new_with_args("bind, id, lookup, get, set, set_val, set_name, set_name_from_hash, push_after", &tkn.value_to_string());
            production.add_attr(hash("action"), Box::new(func.clone()));
            for i in &production_block {
              grammar.production_mut(*i).add_attr(hash("action"), Box::new(func.clone()));
            }
            state = BuildState::EndAction;
          } else if tkn.name() == hash("semicolon") {
            GrammarBuilder::if_empty_error_production_add_e_term(&mut production);
            grammar.push_production(Rc::new(production.clone()));
            production_block.push(grammar.len() - 1);
            state = BuildState::WaitName;
          } else {
            return Err(format!("Unexpected token {}!", tkn.value_to_string()));
          }
        },
        BuildState::EndAction => {
          if tkn.name() == hash("semicolon") {
            GrammarBuilder::if_empty_error_production_add_e_term(&mut production);
            grammar.push_production(Rc::new(production.clone()));
            production_block.push(grammar.len() - 1);
            state = BuildState::WaitName;
          } else {
            return Err(format!("Unexpected token {}!", tkn.value_to_string()));
          }
        }
      }
    }
    Ok(grammar)
  }

}

#[derive(Debug, PartialEq)]
enum Conflict {
  ShiftShift,
  ShiftReduce,
  ReduceReduce
}

fn build_goto_states<T: LRItems>(grammar: &Grammar) -> GotoStatesOpt {
  let mut states = GotoStates::new();
  let canonical = T::canonical(grammar);
  let grammar_symbols = grammar.symbols();
  for i in 0..canonical.len() {
    let items = &canonical[i];
    for symbol in &grammar_symbols {
      let goto = items.goto(grammar, symbol);
      if goto.len() > 0 {
        for j in 0..canonical.len() {
          if canonical[j].contains(&goto) && canonical[j].len() == goto.len() {
            states.set_state(i, symbol.name(), j);
            break;
          }
        }
      }
    }
  }

  GotoStatesOpt::from(&states)
}

pub trait StatesBuilder {
  fn build_goto_states(grammar: &Grammar) -> GotoStatesOpt;

  fn build_collection_items<'a>(grammar: &'a Grammar, goto_states: &'a GotoStatesOpt) -> Vec<LRItems1>;

  fn build_action_states<'a>(grammar: &'a Grammar, goto_states: &'a GotoStatesOpt) -> Result<ActionStatesOpt, String> {
    let lalr = Self::build_collection_items(grammar, goto_states);
    let mut action_states = ActionStates::new();
    let mut conflicts: Vec<(Conflict, Option<Rc<GrammarProduction>>, Rc<GrammarProduction>, usize)> = Vec::new();

    let mut push_if_not_exists = |item: (Conflict, Option<Rc<GrammarProduction>>, Rc<GrammarProduction>, usize)| {
      if !conflicts.iter().any(|i| i.0 == item.0 && i.1 == item.1 && i.2 == item.2 && i.3 == item.3) {
        conflicts.push(item);
      }
    };

    for i in 0..lalr.len() {
      let items = &lalr[i];
      for i1 in 0..items.len() {
        let item = items.item(i1);
        if item.position < item.production.len() {
          let symbol = item.production.symbol(item.position);
          if symbol.is_term() {
            let j = goto_states.state(i, symbol.name());
            if j.is_some() {
              let j = *j.unwrap();
              let state = action_states.state(&i, &symbol.name());
              if state.is_some() {
                let state = state.unwrap();
                if state.state() != &ActionState::Shift {
                  push_if_not_exists((Conflict::ShiftReduce,
                    state.production().clone(), item.production.clone(), symbol.name()));
                } else if state.goto() != &Some(j) {
                  push_if_not_exists((Conflict::ShiftShift,
                    state.production().clone(), item.production.clone(), symbol.name()));
                }
              }

              action_states.set_state(i, symbol.name(),
                ActionState2::new(ActionState::Shift, Some(j), None));
            }
          }
        }

        if item.position > 0 && item.position == item.production.len() &&
          item.production.name() != grammar.production(0).name() {
            let term_name = item.term_name.clone().unwrap();
            let state = action_states.state(&i, &term_name);
            if state.is_some() {
              let state = state.unwrap();
              if state.state() == &ActionState::Shift {
                push_if_not_exists((Conflict::ShiftReduce,
                  state.production().clone(), item.production.clone(), term_name));
              } else if state.production() != &Some(item.production.clone()) {
                push_if_not_exists((Conflict::ReduceReduce,
                  state.production().clone(), item.production.clone(), term_name));
              }
            }

            action_states.set_state(i, term_name,
              ActionState2::new(ActionState::Reduce, None, Some(item.production.clone())));
        }

        let s_term_name = GrammarSymbol::s_term().name();
        if item.position > 0 && item.position == item.production.len() &&
          item.production.as_ref() == grammar.production(0) &&
          item.term_name.clone().unwrap() == s_term_name {
            let state = action_states.state(&i, &s_term_name);
            if state.is_some() {
              let state = state.unwrap();
              if state.state() == &ActionState::Shift {
                push_if_not_exists((Conflict::ShiftReduce,
                  state.production().clone(), item.production.clone(), s_term_name));
              } else if state.production() != &Some(item.production.clone()) {
                push_if_not_exists((Conflict::ReduceReduce,
                  state.production().clone(), item.production.clone(), s_term_name));
              }
            }

            action_states.set_state(i, s_term_name,
              ActionState2::new(ActionState::Accept, None, Some(item.production.clone())));
        }
      }
    }

    if conflicts.len() > 0 {
      if cfg!(debug_assertions) {
        for i in 0..conflicts.len() {
          let conflict = &conflicts[i];
          let msg = match conflict.1 {
            Some(ref prod) =>
              format!("Grammar conflict {:?}, production1: {}, production2: {}, symbol: {:?}",
                conflict.0, prod, conflict.2, get_original_name(conflict.3)),
            _ =>
              format!("Grammar conflict {:?}, production1: {}, production2: {}, symbol: {:?}",
                conflict.0, "", conflict.2, get_original_name(conflict.3)),
          };
          log(&msg);
        }
        log(format!("conflicts count: {}", conflicts.len()).as_str());
      }
      return Err("Grammar is not LALR!".to_string())
    };

    Ok(ActionStatesOpt::from(&action_states))
  }
}

pub struct LALRBuilder {
}

impl LALRBuilder {
  pub fn new() -> Self {
    Self {
    }
  }

  fn make_lr_items1(item: LRItem) -> LRItems1 {
    let mut items = LRItems1::new();
    items.push_item(item);
    items
  }
}

impl StatesBuilder for LALRBuilder {
  fn build_goto_states(grammar: &Grammar) -> GotoStatesOpt {
    build_goto_states::<LRItems0>(grammar)
  }

  fn build_collection_items<'a>(grammar: &'a Grammar, goto_states: &'a GotoStatesOpt) -> Vec<LRItems1> {
    let mut kernels : Vec<LRItems0> = vec![];
    let canonical = LRItems0::canonical(grammar);
    for index in 0..canonical.len() {
      let items = &canonical[index];
      let mut kernel = items.kernel();
      if index == 0 {
        kernel.push_item(LRItem::new(0, grammar.production_clone_rc(0), None));
      }
      kernels.push(kernel);
    }

    let mut lalr_kernels : Vec<LRItems1> = vec![];
    for index in 0..kernels.len() {
      let mut lr1 = LRItems1::new();
      if index == 0 {
        lr1.push_item(LRItem::new(0, grammar.production_clone_rc(0), Some(GrammarSymbol::s_term().name())));
      }
      lalr_kernels.push(lr1);
    }

    let l_term_name = Some(GrammarSymbol::l_term().name());

    loop {
      let mut count = 0;
      for i in 0..kernels.len() {
        let kernel = &kernels[i];
        for i1 in 0..kernel.len() {
          let kern_item = kernel.item(i1);
          let closure = Self::make_lr_items1(
            LRItem::new(
              kern_item.position, kern_item.production.clone(), Some(GrammarSymbol::l_term().name())
            )
          ).closure(grammar);

          for i2 in 0..closure.len() {
            let mut j: Option<&usize> = None;
            let clos_item = closure.item(i2);
            if clos_item.position < clos_item.production.len() {
              let symbol = clos_item.production.symbol(clos_item.position);
              j = goto_states.state(i, symbol.name());
            }
            if j == None {continue};
            let j = *j.unwrap();
            if clos_item.term_name != l_term_name {
              let lr_item = LRItem::new(clos_item.position + 1, clos_item.production.clone(), clos_item.term_name.clone());
              let lr_items = Self::make_lr_items1(lr_item.clone());
              if lalr_kernels.get(j).is_some() {
                if !lalr_kernels[j].contains(&lr_items) {
                  lalr_kernels[j].push_item(lr_item);
                  count += 1;
                }
              }
            } else {
              for i3 in 0..lalr_kernels[i].len() {
                let item = lalr_kernels[i].item(i3);
                if item.production == kern_item.production {
                  if lalr_kernels.get(j).is_some() {
                    let lr_item = LRItem::new(clos_item.position + 1, clos_item.production.clone(), item.term_name.clone());
                    let lr_items = Self::make_lr_items1(lr_item.clone());
                    if !lalr_kernels[j].contains(&lr_items) {
                      lalr_kernels[j].push_item(lr_item);
                      count += 1;
                    }
                  }
                }
              }
            }
          }
        }
      }
      if count == 0 {break};
    }

    let mut result : Vec<LRItems1> = Vec::new();
    for i in 0..lalr_kernels.len() {
      let items = &lalr_kernels[i];
      result.push(items.closure(grammar));
    }
    result
  }
}

pub struct LRBuilder {
}

impl LRBuilder {
  pub fn new() -> Self {
    Self {
    }
  }
}

impl StatesBuilder for LRBuilder {
  fn build_goto_states(grammar: &Grammar) -> GotoStatesOpt {
    build_goto_states::<LRItems1>(grammar)
  }

  fn build_collection_items<'a>(grammar: &'a Grammar, _goto_states: &'a GotoStatesOpt) -> Vec<LRItems1> {
    LRItems1::canonical(grammar)
  }
}