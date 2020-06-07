use std::rc::Rc;
use wasm_bindgen_test::*;

use server::utils::*;

fn create_grammar() -> Grammar {
  let mut grammar = Grammar::new();

  let mut gp = GrammarProduction::new(hash("E1"), None);
  gp.push_symbol(GrammarSymbol::non_term(hash("E"), None));
  grammar.push_production(Rc::new(gp));

  let mut gp = GrammarProduction::new(hash("E"), None);
  gp.push_symbol(GrammarSymbol::non_term(hash("E"), None));
  gp.push_symbol(GrammarSymbol::term(hash("+"), None));
  gp.push_symbol(GrammarSymbol::non_term(hash("T"), None));
  grammar.push_production(Rc::new(gp));

  let mut gp = GrammarProduction::new(hash("E"), None);
  gp.push_symbol(GrammarSymbol::non_term(hash("T"), None));
  grammar.push_production(Rc::new(gp));

  let mut gp = GrammarProduction::new(hash("T"), None);
  gp.push_symbol(GrammarSymbol::non_term(hash("T"), None));
  gp.push_symbol(GrammarSymbol::term(hash("*"), None));
  gp.push_symbol(GrammarSymbol::non_term(hash("F"), None));
  grammar.push_production(Rc::new(gp));

  let mut gp = GrammarProduction::new(hash("T"), None);
  gp.push_symbol(GrammarSymbol::non_term(hash("F"), None));
  grammar.push_production(Rc::new(gp));

  let mut gp = GrammarProduction::new(hash("F"), None);
  gp.push_symbol(GrammarSymbol::term(hash("("), None));
  gp.push_symbol(GrammarSymbol::non_term(hash("E"), None));
  gp.push_symbol(GrammarSymbol::term(hash(")"), None));
  grammar.push_production(Rc::new(gp));

  let mut gp = GrammarProduction::new(hash("F"), None);
  gp.push_symbol(GrammarSymbol::term(hash("id"), None));
  grammar.push_production(Rc::new(gp));

  grammar
}

fn create_grammar1() -> Grammar {
  let mut grammar = Grammar::new();

  let mut gp = GrammarProduction::new(hash("S1"), None);
  gp.push_symbol(GrammarSymbol::non_term(hash("S"), None));
  grammar.push_production(Rc::new(gp));

  let mut gp = GrammarProduction::new(hash("S"), None);
  gp.push_symbol(GrammarSymbol::non_term(hash("C"), None));
  gp.push_symbol(GrammarSymbol::non_term(hash("C"), None));
  grammar.push_production(Rc::new(gp));

  let mut gp = GrammarProduction::new(hash("C"), None);
  gp.push_symbol(GrammarSymbol::term(hash("c"), None));
  gp.push_symbol(GrammarSymbol::non_term(hash("C"), None));
  grammar.push_production(Rc::new(gp));

  let mut gp = GrammarProduction::new(hash("C"), None);
  gp.push_symbol(GrammarSymbol::term(hash("d"), None));
  grammar.push_production(Rc::new(gp));

  grammar
}

#[wasm_bindgen_test]
fn test_lr_items0_canonical() {
  let grammar = create_grammar();

  let canonical = LRItems0::canonical(&grammar);
  assert_eq!(canonical.len(), 12, "Invalid canonical length");

  let lens = vec!(7, 2, 2, 1, 7, 1, 5, 3, 2, 2, 1, 1);
  for i in 0..lens.len() {
    let len = lens[i];
    assert_eq!(canonical[i].len(), len, "Invalid items: '{:?}' length", i);
  };
}

#[wasm_bindgen_test]
fn test_lr_items1_canonical() {
  let grammar = create_grammar1();

  let canonical = LRItems1::canonical(&grammar);
  assert_eq!(canonical.len(), 10, "Invalid canonical length");

  let lens = vec!(6, 1, 3, 6, 2, 1, 3, 1, 2, 1);
  for i in 0..lens.len() {
    let len = lens[i];
    assert_eq!(canonical[i].len(), len, "Invalid items: '{:?}' length", i);
  };
}
