exports.do_lex_action = (func, context, get, set, set_name, set_name_from_hash, pass) => {
  func.call(context, get, set, set_name, set_name_from_hash, pass);
}