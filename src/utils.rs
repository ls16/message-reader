use std::rc::Rc;
use std::sync::Mutex;
use std::collections::{HashSet, HashMap, BTreeMap};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher, BuildHasherDefault};
use std::fmt;
use std::fmt::Debug;
use std::any::{TypeId, Any};
use js_sys::Function;
use wasm_bindgen::prelude::{JsValue, wasm_bindgen};
use lazy_static::*;

#[wasm_bindgen]
extern "C" {
  #[wasm_bindgen(js_namespace = console)]
  pub fn log(s: &str);
  #[wasm_bindgen]
  pub fn perf_now() -> f64;
}

#[derive(Default)]
pub struct UsizeHasher {
  hash: u64
}

impl Hasher for UsizeHasher {
  fn write(&mut self, bytes: &[u8]) {
    self.hash = u32::from_be_bytes([bytes[3], bytes[2], bytes[1], bytes[0]]) as u64;
  }

  fn finish(&self) -> u64 {
    self.hash
  }
}

pub type BuildUsizeHasher = BuildHasherDefault<UsizeHasher>;

lazy_static! {
  static ref NAMES: Mutex<HashMap<u64, usize, BuildUsizeHasher>> = {
      Mutex::new(HashMap::default())
  };
}

lazy_static! {
  static ref ORIGINAL_NAMES: Mutex<HashMap<usize, String, BuildUsizeHasher>> = {
      Mutex::new(HashMap::default())
  };
}

#[wasm_bindgen]
pub fn hash(name: &str) -> usize {
  let mut hasher = DefaultHasher::new();
  hasher.write(name.as_bytes());
  let name_u64 = hasher.finish();

  let mut names = NAMES.lock().unwrap();
  let next_hash = names.len();
  let hash = match names.get(&name_u64) {
    Some(hash) => *hash,
    _ => {
      names.insert(name_u64, next_hash);
      if cfg!(debug_assertions) {
        let mut original_names = ORIGINAL_NAMES.lock().unwrap();
        original_names.insert(next_hash, name.to_string());
      }
      next_hash
    }
  };

  hash as usize
}

#[wasm_bindgen]
pub fn get_original_name(name: usize) -> Option<String> {
  let original_names = ORIGINAL_NAMES.lock().unwrap();
  match original_names.get(&name) {
    Some(original_name) => Some(original_name.clone()),
    _ => None
  }
}

///
/// Token
///
#[derive(Debug, Clone)]
pub struct Token {
  name: usize,
  value: Vec<u8>
}

impl PartialEq for Token {
    fn eq(&self, other: &Token) -> bool {
      if self.name != other.name {return false};
      self.value == other.value
    }
}

impl Eq for Token {}

impl Token {
  /// Creates an instance of Token.
  pub fn new(name: usize, value: Vec<u8>) -> Token {
    Token {
      name,
      value
    }
  }

  /// Gets name of token.
  pub fn name(&self) -> usize {
    self.name
  }

  /// Sets name of token.
  pub fn set_name(&mut self, name_str: &str) {
    self.name = hash(name_str);
  }

  /// Gets value of token.
  pub fn value(&self) -> &Vec<u8> {
    &self.value
  }

  /// Sets value of token.
  pub fn set_value(&mut self, value: Vec<u8>) {
    self.value = value;
  }

  ///Gets value as String
  pub fn value_to_string(&self) -> String {
    String::from_utf8(self.value.clone()).unwrap_or_default()
  }
}

///
/// Type of grammar symbol
/// 
#[derive(Hash, Clone, Copy, PartialEq, Debug)]
pub enum GSType {
  /// Terminal
  T,
  /// Nonterminal
  N
}

///
/// Grammar symbol
///
#[derive(Hash, Debug)]
pub struct GrammarSymbol {
  name: usize,
  value: Option<Vec<u8>>,
  tp: GSType
}

impl PartialEq for GrammarSymbol {
    fn eq(&self, other: &GrammarSymbol) -> bool {
      if self.name != other.name {return false}
      if self.tp != other.tp {return false}
      self.value == other.value
    }
}

impl Eq for GrammarSymbol {}

impl Clone for GrammarSymbol {
  /// Returns a clone of the symbol.
  fn clone(&self) -> GrammarSymbol {
    match &self.value {
      Some(x) => GrammarSymbol {
        name: self.name,
        value: Some(x.clone()),
        tp: self.tp
      },
      None => GrammarSymbol {
        name: self.name,
        value: None,
        tp: self.tp
      },
    }
  }
}

unsafe impl Sync for GrammarSymbol {}

impl GrammarSymbol {
  /// Creates an instance of grammarSymbol.
  pub fn new(name: usize, value: Option<Vec<u8>>, tp: GSType) -> GrammarSymbol {
    GrammarSymbol {
      name,
      value,
      tp
    }
  }

  /// Gets name of grammar symbol.
  pub fn name(&self) -> usize {
    self.name
  }

  /// Changes name of grammar symbol.
  pub fn set_name(&mut self, name: usize) {
    self.name = name;
  }

  /// Gets value of grammar symbol.
  pub fn value(&self) -> &Option<Vec<u8>> {
    &self.value
  }

  /// Takes value of grammar symbol.
  pub fn take_value(&mut self) -> Option<Vec<u8>> {
    self.value.take()
  }

  /// Sets value of grammar symbol.
  pub fn set_value(&mut self, value: Option<Vec<u8>>) {
    self.value = value;
  }

  /// Sets value of grammar symbol.
  pub fn extend_value(&mut self, data: Option<Vec<u8>>) {
    if let Some(data) = data {
      let value = match self.value.take() {
        Some(mut value) => {
          value.extend(data);
          value
        }
        None => data
      };
      self.value = Some(value);
    }
  }

  /// Creates new terminal symbol.
  pub fn term(name: usize, value: Option<Vec<u8>>) -> GrammarSymbol {
    GrammarSymbol {
      name,
      value,
      tp: GSType::T
    }
  }

  /// Gets e-terminal symbol name
  pub fn e_term_name() -> &'static str {
    ""
  }

  /// Gets s-terminal symbol name
  pub fn s_term_name() -> &'static str {
    "$"
  }

  /// Gets #-terminal symbol name
  pub fn l_term_name() -> &'static str {
    "#"
  }

    /// Gets wait-terminal symbol name
    pub fn w_term_name() -> &'static str {
      "w"
    }

  /// Creates new e-terminal symbol.
  pub fn e_term() -> GrammarSymbol {
    GrammarSymbol {
      name: hash(GrammarSymbol::e_term_name()),
      value: None,
      tp: GSType::T
    }
  }

  /// Creates new $-terminal symbol.
  pub fn s_term() -> GrammarSymbol {
    GrammarSymbol {
      name: hash(GrammarSymbol::s_term_name()),
      value: None,
      tp: GSType::T
    }
  }

  /// Creates new #-terminal symbol.
  pub fn l_term() -> GrammarSymbol {
    GrammarSymbol {
      name: hash(GrammarSymbol::l_term_name()),
      value: None,
      tp: GSType::T
    }
  }

  /// Creates new wait-terminal symbol.
  pub fn w_term() -> GrammarSymbol {
    GrammarSymbol {
      name: hash(GrammarSymbol::w_term_name()),
      value: None,
      tp: GSType::T
    }
  }

  /// Creates new nonterminal symbol.
  pub fn non_term(name: usize, value: Option<Vec<u8>>) -> GrammarSymbol {
    GrammarSymbol {
      name,
      value,
      tp: GSType::N
    }
  }

  /// Creates new terminal symbol from token
  pub fn from_token(tkn: Option<Token>) -> GrammarSymbol {
    match tkn {
      Some(tkn) => GrammarSymbol {
        name: tkn.name,
        value: Some(tkn.value),
        tp: GSType::T
      },
      None => GrammarSymbol::s_term()
    }
  }

  /// Returns true if the symbol is a terminal, otherwise false.
  pub fn is_term(&self) -> bool {
    match &self.tp {
      GSType::T => true,
      _ => false
    }
  }

  /// Returns true if the symbol is a e-terminal, otherwise false.
  pub fn is_e_term(&self) -> bool {
    if self.is_term() && self.name() == hash(GrammarSymbol::e_term_name()) {
      true
    } else {
      false
    }
  }

  /// Returns true if the symbol is a $-terminal, otherwise false.
  pub fn is_s_term(&self) -> bool {
    if self.is_term() && self.name() == hash(GrammarSymbol::s_term_name()) {
      true
    } else {
      false
    }
  }

  /// Returns true if the symbol is a #-terminal, otherwise false.
  pub fn is_l_term(&self) -> bool {
    if self.is_term() && self.name() == hash(GrammarSymbol::l_term_name()) {
      true
    } else {
      false
    }
  }

  /// Returns true if the symbol is a wait-terminal, otherwise false.
  pub fn is_w_term(&self) -> bool {
    if self.is_term() && self.name() == hash(GrammarSymbol::w_term_name()) {
      true
    } else {
      false
    }
  }

  /// Returns true if the symbol is a nonterminal, otherwise false.
  pub fn is_non_term(&self) -> bool {
    match &self.tp {
      GSType::N => true,
      _ => false
    }
  }

}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum RustActionCommand {
  Set
}

#[derive(Clone, Debug)]
pub struct RustAction {
  command: RustActionCommand,
  index: usize,
  index2: Option<usize>,
  index3: Option<usize>,
  index4: Option<usize>,
  index5: Option<usize>
}

impl RustAction {
  pub fn new(command: RustActionCommand, index: usize, index2: Option<usize>, index3: Option<usize>,
    index4: Option<usize>, index5: Option<usize>) -> Self {
    Self {
      command,
      index,
      index2,
      index3,
      index4,
      index5
    }
  }

  pub fn command(&self) -> RustActionCommand {
    self.command
  }

  pub fn index(&self) -> usize {
    self.index
  }

  pub fn index2(&self) -> Option<usize> {
    self.index2
  }

  pub fn index3(&self) -> Option<usize> {
    self.index3
  }

  pub fn index4(&self) -> Option<usize> {
    self.index4
  }

  pub fn index5(&self) -> Option<usize> {
    self.index5
  }
}

pub trait Attribute: Any {
  fn as_function(&self) -> Option<&Function> {
    None
  }

  fn as_vec(&self) -> Option<&Vec<u8>> {
    None
  }

  fn as_string(&self) -> Option<&String> {
    None
  }

  fn as_usize(&self) -> Option<usize> {
    None
  }

  fn as_rust_action(&self) -> Option<&RustAction> {
    None
  }
}

impl Debug for dyn Attribute {
  fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
    Ok(())
  }  
}

impl Attribute for Function {
  fn as_function(&self) -> Option<&Function> {
    Some(self)
  }
}

impl Attribute for Vec<u8> {
  fn as_vec(&self) -> Option<&Vec<u8>> {
    Some(self)
  }
}

impl Attribute for String {
  fn as_string(&self) -> Option<&String> {
    Some(self)
  }
}

impl Attribute for usize {
  fn as_usize(&self) -> Option<usize> {
    Some(*self)
  }
}

impl Attribute for RustAction {
  fn as_rust_action(&self) -> Option<&RustAction> {
    Some(self)
  }
}


///
/// Attributes
///
#[derive(Debug)]
pub struct Attributes {
  pub attrs: HashMap<usize, Box<dyn Attribute>, BuildUsizeHasher>
}

impl Clone for Attributes {
  fn clone(&self) -> Self {
    let mut attrs = Attributes::new();
    for (name, attr) in &self.attrs {
      let type_id = attr.as_ref().type_id();
      if type_id == TypeId::of::<Function>() {
        attrs.insert(*name, Box::new(attr.as_function().unwrap().clone()));
      } else if type_id == TypeId::of::<Vec<u8>>() {
        attrs.insert(*name, Box::new(attr.as_vec().unwrap().clone()));
      } else if type_id == TypeId::of::<String>() {
        attrs.insert(*name, Box::new(attr.as_string().unwrap().clone()));
      } else if type_id == TypeId::of::<RustAction>() {
        attrs.insert(*name, Box::new(attr.as_rust_action().unwrap().clone()));
      }
    }
    attrs
  }
}

unsafe impl Sync for Attributes {}

impl Attributes {
  pub fn new() -> Attributes {
    Attributes {
      attrs: HashMap::default()
    }
  }

  pub fn len(&self) -> usize {
    self.attrs.len()
  }

  pub fn get(&self, key: &usize) -> Option<&Box<dyn Attribute>> {
    self.attrs.get(key)
  }

  pub fn insert(&mut self, key: usize, attr: Box<dyn Attribute>) {
    self.attrs.insert(key, attr);
  }

  pub fn remove(&mut self, key: &usize) -> Option<Box<dyn Attribute>> {
    self.attrs.remove(key)
  }

  pub fn keys(&self) -> HashSet<usize> {
    let mut keys: HashSet<usize> = HashSet::new();
    for key in self.attrs.keys() {
      keys.insert(*key);
    }
    keys
  }
}

///
/// Grammar production
///
#[derive(Debug)]
pub struct GrammarProduction {
  name: usize,
  symbols: Vec<GrammarSymbol>,
  attrs: Attributes
}

impl fmt::Display for GrammarProduction {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let prod_name = match get_original_name(self.name) {
      Some(name) => name,
      _ => "".to_string()
    };
    let _ = write!(f, "{}:", prod_name);
    for i in 0..self.symbols.len() {
      let symbol = &self.symbols[i];
      let sym_name = match get_original_name(symbol.name()) {
        Some(name) => name,
        _ => "".to_string()
      };
      let sym_name = match symbol.is_term() {
        true => format!("'{}'", sym_name),
        false => sym_name
      };
      let _ = write!(f, " {}", sym_name);
    }
    Ok(())
  }
}

impl PartialEq for GrammarProduction {
    fn eq(&self, other: &GrammarProduction) -> bool {
        if self.name != other.name {return false};
        self.symbols == other.symbols
    }
}

impl Eq for GrammarProduction {}

impl Clone for GrammarProduction {
  fn clone(&self) -> Self {
    let mut prod = GrammarProduction::new(self.name.clone(), Some(self.attrs.clone()));
    for symbol in &self.symbols {
      prod.push_symbol(symbol.clone());
    }
    prod
  }
}

impl GrammarProduction {
  pub fn new(name: usize, attrs: Option<Attributes>) -> GrammarProduction {
    GrammarProduction {
      name,
      symbols: vec![],
      attrs: match attrs {
        Some(data) => data,
        None => Attributes::new()
      }
    }
  }

  pub fn name(&self) -> usize {
    self.name
  }

  pub fn len(&self) -> usize {
    self.symbols.len()
  }

  pub fn symbol(&self, index: usize) -> &GrammarSymbol {
    &self.symbols[index]
  }

  pub fn push_symbol(&mut self, symbol: GrammarSymbol) {
    self.symbols.push(symbol);
  }

  pub fn remove_symbol(&mut self, index: usize) {
    self.symbols.remove(index);
  }

  pub fn add_attr(&mut self, key: usize, attr: Box<dyn Attribute>) {
    self.attrs.insert(key, attr);
  }

  pub fn attr(&self, key: &usize) -> Option<&Box<dyn Attribute>> {
    self.attrs.get(key)
  }

  pub fn find(&self, name: usize) -> Vec<usize> {
    let mut result: Vec<usize> = Vec::new();
    for i in 0..self.symbols.len() {
      let sym = &self.symbols[i];
      if sym.name == name {
        result.push(i);
      }
    }
    result
  }
}

///
/// Grammar
///
#[derive(Debug, Clone)]
pub struct Grammar {
  productions: Vec<Rc<GrammarProduction>>
}

impl Grammar {
  pub fn new() -> Grammar {
    Grammar {
      productions: Vec::new()
    }
  }

  pub fn len(&self) -> usize {
    self.productions.len()
  }

  pub fn production(&self, index: usize) -> &GrammarProduction {
    &self.productions[index]
  }

  pub fn production_clone_rc(&self, index: usize) -> Rc<GrammarProduction> {
    self.productions[index].clone()
  }

  pub fn production_mut(&mut self, index: usize) -> &mut GrammarProduction {
    Rc::get_mut(&mut self.productions[index]).unwrap()
  }

  pub fn push_production(&mut self, production: Rc<GrammarProduction>) {
    self.productions.push(production);
  }

  pub fn remove_production(&mut self, index: usize) {
    self.productions.remove(index);
  }

  pub fn symbols(&self) -> Vec<GrammarSymbol> {
    let mut result = Vec::new();
    let mut exists: HashSet<usize> = HashSet::new();
    for production in &self.productions {
      if !exists.contains(&production.name()) {
        result.push(GrammarSymbol::non_term(production.name(), None));
        exists.insert(production.name());
      }
      for symbol in &production.symbols {
        if !exists.contains(&symbol.name()) {
          result.push(symbol.clone());
          exists.insert(symbol.name());
        }
      }
    }
    result
  }

  fn first1(&self, symbol: &GrammarSymbol, visited: &mut HashSet<usize>, added_names: &mut HashSet<usize>, result: &mut Vec<GrammarSymbol>) {
    let e_term = GrammarSymbol::e_term();
    for pos in 0..self.productions.len() {
      let production = &self.productions[pos];
      if production.name != symbol.name {continue};
      if visited.contains(&pos) {continue};

      visited.insert(pos);
      if production.symbols.len() > 0 {
        for sym in &production.symbols {
          if sym.is_term() {
            if !added_names.contains(&sym.name) {
              result.push(sym.clone());
              added_names.insert(sym.name);
            }
            break;
          }

          self.first1(&sym, visited, added_names, result);
          if result.len() > 0 && result[0].name != e_term.name {break};
        }
      } else {
        if !added_names.contains(&e_term.name) {
          result.push(e_term.clone());
          added_names.insert(e_term.name);
        }
      }
    }
  }

  pub fn first(&self, symbols: &Vec<GrammarSymbol>) -> Vec<GrammarSymbol> {
    let mut visited: HashSet<usize> = HashSet::new();
    let mut added_names: HashSet<usize> = HashSet::new();
    let mut result: Vec<GrammarSymbol> = Vec::new();

    if symbols.len() == 1 {
      let symbol = &symbols[0];
      if symbol.is_term() {
        result.push(symbol.clone());
      } else {
        self.first1(symbol, &mut visited, &mut added_names, &mut result);
      }
    } else {
      let mut has_e_term_all = true;
      for pos in 0..symbols.len() {
        let result1 = self.first(&symbols[pos..pos+1].to_vec());
        let mut has_e_term = false;
        for sym in &result1 {
          if !added_names.contains(&sym.name) {
            result.push(sym.clone());
            added_names.insert(sym.name);
          }
          if sym.is_e_term() {has_e_term = true};
        }
        if !has_e_term {
          has_e_term_all = false;
          break;
        }
      }

      let e_term = GrammarSymbol::e_term();
      if has_e_term_all && !added_names.contains(&e_term.name) {
        result.push(e_term);
      }
    }

    result
  }

}


///
/// Item of LR grammar
///
#[derive(Debug)]
pub struct LRItem {
  pub position: usize,
  pub production: Rc<GrammarProduction>,
  pub term_name: Option<usize>
}

impl PartialEq for LRItem {
    fn eq(&self, other: &LRItem) -> bool {
      self.position == other.position &&
      self.production == other.production &&
      self.term_name == other.term_name
    }
}

impl Eq for LRItem {}

impl Clone for LRItem {
  fn clone(&self) -> Self {
    LRItem {
      position: self.position,
      production: self.production.clone(),
      term_name: self.term_name
    }
  }
}

impl LRItem {
  pub fn new(position: usize, production: Rc<GrammarProduction>, term_name: Option<usize>) -> LRItem {
    LRItem  {
      position,
      production,
      term_name
    }
  }
}

pub trait LRItems : Sized {
  fn new() -> Self;

  fn item(&self, index: usize) -> &LRItem;

  fn len(&self) -> usize;

  fn push_item(&mut self, item: LRItem);

  fn pop_item(&mut self) -> Option<LRItem>;

  fn closure(&self, grammar: &Grammar) -> Self;

  fn clone(&self) -> Self {
    let mut result = Self::new();
    for i in 0..self.len() {
      let itm = (*self.item(i)).clone();
      result.push_item(itm);
    }
    result
  }

  fn extend(&mut self, items: Self) {
    for i in 0..items.len() {
      self.push_item(items.item(i).clone());
    }
  }

  fn contains(&self, items: &Self) -> bool {
    for i in 0..items.len() {
      let mut eq = false;
      for j in 0..self.len() {
        if items.item(i) == self.item(j) {
          eq = true;
          break;
        }
      }
      if !eq {return false;}
    }
    true
  }

  fn kernel(&self) -> Self {
    let mut result = Self::new();
    for i in 0..self.len() {
      if self.item(i).position > 0 {
        result.push_item(self.item(i).clone());
      }
    }
    result
  }

  fn goto(&self, grammar: &Grammar, symbol: &GrammarSymbol) -> Self {
    let mut result = Self::new();
    for i in 0..self.len() {
      let item = self.item(i);
      let positions = item.production.find(symbol.name());
      for j in 0..positions.len() {
        if positions[j] == item.position {
          result.push_item(LRItem::new(
           positions[j] + 1,
           item.production.clone(),
           item.term_name
          ));
        }
      }
    }
    result.closure(grammar)
  }

  fn canonical(grammar: &Grammar) -> Vec<Self> {
    let mut result: Vec<Self> = Vec::new();
    let symbols = grammar.symbols();
    let mut items = Self::new();
    items.push_item(
      LRItem::new(0,
        grammar.production_clone_rc(0),
        Some(GrammarSymbol::s_term().name()))
    );
    result.push(items.closure(&grammar));

    loop {
      let mut result1: Vec<Self> = Vec::new();
      for i in 0..result.len() {
        let items = &result[i];
        for symbol in &symbols {
          let goto_items = items.goto(grammar, symbol);
          if goto_items.len() > 0 && result.iter().find(|items| {
              (items.len() == goto_items.len()) && items.contains(&goto_items)
            }).is_none() {
            result1.push(goto_items);
          }
        }
      }
      if result1.len() == 0 {break};
      result.extend(result1);
    }

    result
  }
}

///
/// Array of LR(0) grammar items.
///
#[derive(Debug)]
pub struct LRItems0 {
  items: Vec<LRItem>
}

impl LRItems for LRItems0 {
  fn new() -> Self {
    Self {
      items: vec![]
    }
  }

  fn item(&self, index: usize) -> &LRItem {
    &self.items[index]
  }

  fn len(&self) -> usize {
    self.items.len()
  }

  fn push_item(&mut self, item: LRItem) {
    self.items.push(item);
  }

  fn pop_item(&mut self) -> Option<LRItem> {
    self.items.pop()
  }

  fn closure(&self, grammar: &Grammar) -> Self {
    let mut result = self.clone();
    let mut added: HashSet<usize> = HashSet::new();
    loop {
      let mut result1 = Self::new();
      for i in 0..result.len() {
        let item = result.item(i);
        if item.position >= item.production.len() {continue};
        let symbol = &item.production.symbols[item.position];
        for pos in 0..grammar.len() {
          let prod = grammar.production(pos);
          if (symbol.name == prod.name()) && !added.contains(&pos) {
            result1.push_item(LRItem::new(0, grammar.production_clone_rc(pos), None));
            added.insert(pos);
          }
        }
      }
      if result1.len() == 0 {break};
      result.extend(result1);
    }
    result
  }

}


//
// Array of LR(1) grammar items.
//
#[derive(Debug)]
pub struct LRItems1 {
  items: Vec<LRItem>
}

impl LRItems for LRItems1 {
  fn new() -> Self {
    Self {
      items: vec![]
    }
  }

  fn item(&self, index: usize) -> &LRItem {
    &self.items[index]
  }

  fn len(&self) -> usize {
    self.items.len()
  }

  fn push_item(&mut self, item: LRItem) {
    self.items.push(item);
  }

  fn pop_item(&mut self) -> Option<LRItem> {
    self.items.pop()
  }

  fn closure(&self, grammar: &Grammar) -> Self {
    let mut result = self.clone();
    let mut added: HashMap<usize, HashSet<usize>> = HashMap::new();
    loop {
      let mut result1 = Self::new();
      for i in 0..result.len() {
        let item = result.item(i);
        if item.position >= item.production.len() {continue};
        let symbol = &item.production.symbols[item.position];
        let mut symbols = Vec::from(&item.production.symbols[(item.position + 1)..]);
        if item.term_name.is_some() {
          symbols.push(GrammarSymbol::term(item.term_name.clone().unwrap(), None));
        }
        if symbols.len() == 0 {
          symbols.push(GrammarSymbol::e_term());
        }
        let first = grammar.first(&symbols);
        for pos in 0..grammar.len() {
          let prod = grammar.production(pos);
          if symbol.name == prod.name() {
            if !added.contains_key(&pos) {added.insert(pos, HashSet::new());};
            for first_symbol in &first {
              if !added.get(&pos).unwrap().contains(&first_symbol.name) {
                result1.push_item(LRItem::new(0, grammar.production_clone_rc(pos), Some(first_symbol.name)));
                added.get_mut(&pos).unwrap().insert(first_symbol.name);
              }
            }
          }
        }
      }
      if result1.len() == 0 {break};
      result.extend(result1);
    }
    result
  }

}

static mut _ID: usize = 0;

fn get_id() -> usize {
  unsafe {
    _ID = _ID + 1;
    _ID
  }
}

#[derive(PartialEq, Debug)]
pub enum ASTItemType {
  Leaf,
  Node
}

pub trait ASTItem {
  fn id(&self) -> usize;
  fn name(&self) -> &str;
  fn type_item(&self) -> ASTItemType;
  fn left(&self) -> Option<Rc<dyn ASTItem>>;
  fn right(&self) -> Option<Rc<dyn ASTItem>>;
  fn attrs_len(&self) -> usize;
  fn attr(&self, key: &usize) -> Option<&Box<dyn Attribute>>;
  fn insert_attr(&mut self, key: usize, attr: Box<dyn Attribute>);
  fn remove_attr(&mut self, key: &usize) -> Option<Box<dyn Attribute>>;
  fn value(&self) -> &Vec<u8>;
}

impl PartialEq for dyn ASTItem {
  fn eq(&self, other: &dyn ASTItem) -> bool {
      self.id() == other.id()
  }
}

impl Eq for dyn ASTItem {}

impl Hash for dyn ASTItem {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

pub struct ASTLeaf {
  id: usize,
  name: String,
  attrs: Attributes,
  value: Vec<u8>
}

impl ASTItem for ASTLeaf {
  fn id(&self) -> usize {
    self.id
  }

  fn name(&self) -> &str {
    &self.name
  }

  fn type_item(&self) -> ASTItemType {
    ASTItemType::Leaf
  }

  fn left(&self) -> Option<Rc<dyn ASTItem>> {
    None
  }

  fn right(&self) -> Option<Rc<dyn ASTItem>> {
    None
  }

  fn attrs_len(&self) -> usize {
    self.attrs.len()
  }

  fn attr(&self, key: &usize) -> Option<&Box<dyn Attribute>> {
    self.attrs.get(key)
  }

  fn insert_attr(&mut self, key: usize, attr: Box<dyn Attribute>) {
    self.attrs.insert(key, attr)
  }

  fn remove_attr(&mut self, key: &usize) -> Option<Box<dyn Attribute>> {
    self.attrs.remove(key)
  }

  fn value(&self) -> &Vec<u8> {
    &self.value
  }
}

impl ASTLeaf {
  pub fn new(name: String, value: Vec<u8>, attrs: Option<Attributes>) -> Self {
    let attrs = match attrs {
      Some(attrs) => attrs,
      None => Attributes::new()
    };
    Self {
      id: get_id(),
      name,
      attrs,
      value
    }
  }

  #[allow(dead_code)]
  fn value(&self) -> &Vec<u8> {
    &self.value
  }
}

pub struct ASTNode {
  id: usize,
  name: String,
  attrs: Attributes,
  left: Rc<dyn ASTItem>,
  right: Option<Rc<dyn ASTItem>>
}

impl ASTItem for ASTNode {
  fn id(&self) -> usize {
    self.id
  }

  fn name(&self) -> &str {
    &self.name
  }

  fn type_item(&self) -> ASTItemType {
    ASTItemType::Node
  }

  fn left(&self) -> Option<Rc<dyn ASTItem>> {
    Some(self.left.clone())
  }

  fn right(&self) -> Option<Rc<dyn ASTItem>> {
    match self.right {
      Some(ref data) => Some(data.clone()),
      _ => None
    }
  }

  fn attrs_len(&self) -> usize {
    self.attrs.len()
  }

  fn attr(&self, key: &usize) -> Option<&Box<dyn Attribute>> {
    self.attrs.get(key)
  }

  fn insert_attr(&mut self, key: usize, attr: Box<dyn Attribute>) {
    self.attrs.insert(key, attr)
  }

  fn remove_attr(&mut self, key: &usize) -> Option<Box<dyn Attribute>> {
    self.attrs.remove(key)
  }

  fn value(&self) -> &Vec<u8> {
    panic!("Not implemented");
  }
}

impl ASTNode {
  pub fn new(name:String, left: Rc<dyn ASTItem>, right: Option<Rc<dyn ASTItem>>, attrs: Option<Attributes>) -> Self {
    let attrs = match attrs {
      Some(attrs) => attrs,
      None => Attributes::new()
    };
    Self {
      id: get_id(),
      name,
      attrs,
      left,
      right
    }
  }
}

#[derive(Clone)]
pub struct ASTBuilder {
  last_id: Option<usize>,
  index: HashMap<usize, Rc<dyn ASTItem>>,
}

impl ASTBuilder {
  pub fn new() -> Self {
    Self {
      last_id: None,
      index: HashMap::new(),
    }
  }

  pub fn last(&self) -> Option<Rc<dyn ASTItem>> {
    match self.last_id {
      Some(ref id) => self.index.get(id).cloned(),
      None => None
    }
  }

  pub fn by_id(&self, id: usize) -> Option<Rc<dyn ASTItem>> {
    self.index.get(&id).cloned()
  }

  fn set_last_id(&mut self, value: Option<usize>) {
    self.last_id = value;
  }

  fn insert_index(&mut self, key: usize, value: Rc<dyn ASTItem>) {
    self.index.insert(key, value);
  }

  pub fn items(&self) -> BTreeMap<usize, Rc<dyn ASTItem>> {
    let mut result: BTreeMap<usize, Rc<dyn ASTItem>> = BTreeMap::new();
    for id in self.index.keys()  {
      result.insert(*id, self.index.get(&id).cloned().unwrap());
    }
    result
  }


  pub fn add_leaf(&mut self, name: String, value: Vec<u8>,
    attrs: Option<Attributes>) -> Result<Rc<dyn ASTItem>, JsValue> {
    let leaf: Rc<dyn ASTItem> = Rc::new(ASTLeaf::new(name, value, attrs));
    let id = leaf.id();
    self.insert_index(id, leaf);
    Ok(self.index.get(&id).cloned().unwrap())
  }

  pub fn add_leaf_id(&mut self, name: String, value: Vec<u8>,
    attrs: Option<Attributes>) -> Result<usize, JsValue> {
    let leaf = self.add_leaf(name, value, attrs)?;
    Ok(leaf.id())
  }

  pub fn add_node(&mut self, name: String, left: Rc<dyn ASTItem>, right: Option<Rc<dyn ASTItem>>,
    attrs: Option<Attributes>) -> Result<Rc<dyn ASTItem>, JsValue> {
    let node: Rc<dyn ASTItem> = Rc::new(ASTNode::new(name, left.clone(), right.clone(), attrs));
    let id = node.id();
    self.set_last_id(Some(id));
    self.insert_index(id, node);
    Ok(self.index.get(&id).cloned().unwrap())
  }

  pub fn add_node_id(&mut self, name: String, left_id: usize, right_id: Option<usize>,
    attrs: Option<Attributes>) -> Result<usize, JsValue> {
      let left = self.by_id(left_id);
      if left.is_none() {return Err(JsValue::from(format!("Unknown left id: {}", left_id)));}
      let left = left.unwrap();
      let right = match right_id {
        Some(right_id) => {
          let right = self.by_id(right_id);
          if right.is_none() {return Err(JsValue::from(format!("Unknown right id: {}", right_id)));}
          right
        },
        _ => None
      };
      let node = self.add_node(name, left, right, attrs)?;
      Ok(node.id())
  }
}

pub trait ParserExecContext {
  fn builder(&mut self) -> &mut ASTBuilder;
}

impl Debug for dyn ParserExecContext {
  fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
    Ok(())
  }
}

impl ParserExecContext for JsValue {
  fn builder(&mut self) -> &mut ASTBuilder {
    panic!("Not implemented")
  }
}
